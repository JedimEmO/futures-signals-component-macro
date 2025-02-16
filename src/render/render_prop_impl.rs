use crate::parse::{Component, Prop, SignalType};
use crate::render::render_doc_exprs;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Token, Type, TypeImplTrait};

pub fn render_prop_impl(props_struct_name: &Ident, prop: &Prop, cmp: &Component) -> TokenStream {
    let prop_name = &prop.name;
    let ty_ = prop.type_.clone();

    // If this is a dyn T object, convert it to impl T, otherwise transparent
    let arg_type = match prop.type_.clone() {
        Type::TraitObject(to) => Type::ImplTrait(TypeImplTrait {
            bounds: to.bounds,
            impl_token: Token![impl](props_struct_name.span()),
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

    let value_assign_expr = if let Some(_default) = &prop.default {
        quote! {v}
    } else {
        quote! {Some(v)}
    };

    let rest_of_props = cmp.props.iter().filter(|p| p.name != prop.name).map(|p| {
        let name = &p.name;

        quote! {
            #name: self.#name,
        }
    });

    if prop.is_signal.is_some() {
        let props_signal_fn_name = match prop.is_signal.as_ref().unwrap() {
            SignalType::Item => syn::parse_str::<Ident>(format!("{}_signal", prop.name).as_str())
                .expect("failed to parse props signal fn name"),
            SignalType::Vec => {
                syn::parse_str::<Ident>(format!("{}_signal_vec", prop.name).as_str())
                    .expect("failed to parse props signal fn name")
            }
        };

        let signal_mod_ident = match prop.is_signal.as_ref().unwrap() {
            SignalType::Item => Ident::new("signal", prop.type_.span()),
            SignalType::Vec => Ident::new("signal_vec", prop.type_.span()),
        };

        let always_arg_type = match prop.is_signal.as_ref().unwrap() {
            SignalType::Item => quote! {#arg_type},
            SignalType::Vec => quote! {impl Into<Vec<#value_type>>},
        };

        let send_constraint = if prop.is_send {
            quote! { + Send }
        } else {
            quote! {}
        };

        let signal_type = match prop.is_signal.as_ref().unwrap() {
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

        quote! {
            impl #props_struct_name {
                #docs
                pub fn #prop_name(mut self, v: #always_arg_type) -> #props_struct_name {
                    self.#props_signal_fn_name(futures_signals::#signal_mod_ident::always(#value_constructor #vec_into))
                }

                #docs
                pub fn #props_signal_fn_name(self, v: #signal_type) -> #props_struct_name {
                    use futures_signals::signal::SignalExt;
                    use futures_signals::signal_vec::SignalVecExt;

                    #box_statement

                    #props_struct_name {
                        #prop_name: #value_assign_expr,
                        #(#rest_of_props)*
                    }
                }
            }
        }
    } else {
        quote! {
            impl #props_struct_name {
                #docs
                pub fn #prop_name(mut self, v: #arg_type) -> #props_struct_name {
                    let v = #value_constructor;

                     #props_struct_name {
                        #prop_name: #value_assign_expr,
                        #(#rest_of_props)*
                    }
                }
            }
        }
    }
}
