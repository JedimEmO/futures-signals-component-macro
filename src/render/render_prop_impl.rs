use crate::parse::{Component, Prop, SignalType};
use crate::render::render_doc_exprs;
use crate::render::render_utils::{compute_prop_type_ident, BuilderCtx};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Token, Type, TypeImplTrait};

pub fn render_prop_impl(ctx: &BuilderCtx, prop: &Prop, cmp: &Component) -> TokenStream {
    let prop_name = &prop.name;
    let target = &ctx.setter_target;

    let impl_generics = ctx.impl_generics();
    let ty_generics = ctx.ty_generics();

    // Setting a required prop flips its type parameter to the concrete stored type; all
    // other setters leave the parameterization untouched.
    let ret_generics = if prop.is_required {
        ctx.ty_generics_with(prop_name)
    } else {
        ty_generics.clone()
    };

    // If this is a dyn T object, convert it to impl T, otherwise transparent
    let arg_type = match prop.type_.clone() {
        Type::TraitObject(to) => Type::ImplTrait(TypeImplTrait {
            bounds: to.bounds,
            impl_token: Token![impl](ctx.props_name.span()),
        }),
        ty_ => ty_,
    };

    // if dyn object, Box<dyn T>, otherwise T
    let value_type = match prop.type_.clone() {
        Type::TraitObject(to) => {
            quote! {
                Box<#to>
            }
        }
        ty_ => quote! { #ty_ },
    };

    // if dyn object, Box::new(v), otherwise v
    let value_constructor = match prop.type_.clone() {
        Type::TraitObject(to) => {
            quote! {
                { let v: Box<#to> = Box::new(v); v }
            }
        }
        _ => {
            quote! {
                v
            }
        }
    };

    let docs = render_doc_exprs(&prop.docs);

    let value_assign_expr = if prop.default.is_some() || prop.is_required {
        quote! {v}
    } else {
        quote! {Some(v)}
    };

    let rest_of_props = cmp
        .props
        .iter()
        .filter(|p| p.name != prop.name)
        .map(|p| {
            let name = &p.name;

            quote! {
                #name: self.#name,
            }
        })
        .collect::<Vec<_>>();

    if prop.is_compose_apply {
        let to = match &prop.type_ {
            Type::TraitObject(to) => to,
            _ => unreachable!("the injected apply prop is always a trait object"),
        };

        return quote! {
            impl #impl_generics #target #ty_generics {
                #docs
                pub fn #prop_name(self, v: #arg_type) -> #target #ty_generics {
                    let v: Option<Box<#to>> = match self.#prop_name {
                        Some(prev) => Some(Box::new(move |b| v(prev(b)))),
                        None => Some(Box::new(v)),
                    };

                    #target {
                        #prop_name: v,
                        #(#rest_of_props)*
                    }
                }
            }
        };
    }

    if let Some(signal_type_kind) = &prop.is_signal {
        let props_signal_fn_name = match signal_type_kind {
            SignalType::Item => syn::parse_str::<Ident>(format!("{}_signal", prop.name).as_str())
                .expect("failed to parse props signal fn name"),
            SignalType::Vec => {
                syn::parse_str::<Ident>(format!("{}_signal_vec", prop.name).as_str())
                    .expect("failed to parse props signal fn name")
            }
        };

        let signal_mod_ident = match signal_type_kind {
            SignalType::Item => Ident::new("signal", prop.type_.span()),
            SignalType::Vec => Ident::new("signal_vec", prop.type_.span()),
        };

        let always_arg_type = match signal_type_kind {
            SignalType::Item => {
                if prop.is_into {
                    quote! {impl ::core::convert::Into<#value_type>}
                } else {
                    quote! {#arg_type}
                }
            }
            SignalType::Vec => quote! {impl Into<Vec<#value_type>>},
        };

        let send_constraint = if prop.is_send {
            quote! { + Send }
        } else {
            quote! {}
        };

        let signal_type = match signal_type_kind {
            SignalType::Item => {
                quote! {impl futures_signals::signal::Signal<Item=#value_type> #send_constraint + 'static}
            }
            SignalType::Vec => {
                quote! {impl futures_signals::signal_vec::SignalVec<Item=#value_type> #send_constraint + 'static}
            }
        };

        let vec_into = match prop.is_signal.as_ref() {
            Some(SignalType::Vec) => quote! { .into() },
            _ => quote! {},
        };

        let box_statement = if prop.is_send {
            quote! { let v = v.boxed(); }
        } else {
            quote! { let v = v.boxed_local(); }
        };

        // The value setter pins the conversion target before wrapping in `always`, so type
        // inference never has to pick between `T` and the signal setter's item parameter.
        let value_setter_body = if prop.is_into {
            quote! {
                let v: #value_type = v.into();
                self.#props_signal_fn_name(futures_signals::signal::always(v))
            }
        } else {
            quote! {
                self.#props_signal_fn_name(futures_signals::#signal_mod_ident::always(#value_constructor #vec_into))
            }
        };

        let signal_setter = if prop.is_into {
            let stored_type = compute_prop_type_ident(prop);
            let box_method = if prop.is_send {
                quote! { boxed() }
            } else {
                quote! { boxed_local() }
            };

            quote! {
                #docs
                pub fn #props_signal_fn_name<TIntoItem: ::core::convert::Into<#value_type> + 'static>(
                    self,
                    v: impl futures_signals::signal::Signal<Item = TIntoItem> #send_constraint + 'static,
                ) -> #target #ret_generics {
                    use futures_signals::signal::SignalExt;

                    let v: #stored_type = v.map(::core::convert::Into::into).#box_method;

                    #target {
                        #prop_name: #value_assign_expr,
                        #(#rest_of_props)*
                    }
                }
            }
        } else {
            quote! {
                #docs
                pub fn #props_signal_fn_name(self, v: #signal_type) -> #target #ret_generics {
                    use futures_signals::signal::SignalExt;
                    use futures_signals::signal_vec::SignalVecExt;

                    #box_statement

                    #target {
                        #prop_name: #value_assign_expr,
                        #(#rest_of_props)*
                    }
                }
            }
        };

        quote! {
            impl #impl_generics #target #ty_generics {
                #docs
                pub fn #prop_name(self, v: #always_arg_type) -> #target #ret_generics {
                    #value_setter_body
                }

                #signal_setter
            }
        }
    } else {
        let (setter_arg_type, setter_value_statement) = if prop.is_into {
            (
                quote! {impl ::core::convert::Into<#value_type>},
                quote! { let v: #value_type = v.into(); },
            )
        } else {
            (quote! {#arg_type}, quote! { let v = #value_constructor; })
        };

        quote! {
            impl #impl_generics #target #ty_generics {
                #docs
                pub fn #prop_name(self, v: #setter_arg_type) -> #target #ret_generics {
                    #setter_value_statement

                    #target {
                        #prop_name: #value_assign_expr,
                        #(#rest_of_props)*
                    }
                }
            }
        }
    }
}
