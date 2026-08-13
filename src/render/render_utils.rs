use crate::parse::{Component, Prop, SignalType};
use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Type;

pub struct RequiredProp {
    pub field: Ident,
    /// The builder type parameter standing in for this prop, e.g. `TContent`
    pub param: Ident,
    /// The marker unit struct stored while the prop is unset, e.g. `CardPropsBuilderMissingContent`
    pub marker: Ident,
    /// The concrete type the prop is stored as once set
    pub stored_type: TokenStream,
}

/// Per-component context for the typestate builder split.
///
/// Components without `#[required]` props keep the 0.4 shape: setters live on the props
/// struct itself and every generics accessor renders empty. Components with required props
/// get a separate `{Props}Builder` type carrying one type parameter per required prop.
pub struct BuilderCtx {
    pub props_name: Ident,
    pub builder_name: Ident,
    pub complete_trait: Ident,
    /// The type the setter impls target: the props struct itself (no required props) or the builder
    pub setter_target: Ident,
    pub required: Vec<RequiredProp>,
}

impl BuilderCtx {
    pub fn new(cmp: &Component) -> Self {
        let props_name = Ident::new(&format!("{}Props", cmp.name), cmp.name.span());
        let builder_name = Ident::new(&format!("{}PropsBuilder", cmp.name), cmp.name.span());
        let complete_trait = Ident::new(
            &format!("{}PropsBuilderComplete", cmp.name),
            cmp.name.span(),
        );

        let required = cmp
            .props
            .iter()
            .filter(|p| p.is_required)
            .map(|p| {
                let pascal = p.name.to_string().to_case(Case::Pascal);

                RequiredProp {
                    field: p.name.clone(),
                    param: Ident::new(&format!("T{pascal}"), p.name.span()),
                    marker: Ident::new(
                        &format!("{}PropsBuilderMissing{pascal}", cmp.name),
                        p.name.span(),
                    ),
                    stored_type: compute_prop_type_ident(p),
                }
            })
            .collect::<Vec<_>>();

        let setter_target = if required.is_empty() {
            props_name.clone()
        } else {
            builder_name.clone()
        };

        Self {
            props_name,
            builder_name,
            complete_trait,
            setter_target,
            required,
        }
    }

    pub fn has_required(&self) -> bool {
        !self.required.is_empty()
    }

    /// `<TContent, TLabel>` — or empty when there are no required props
    pub fn impl_generics(&self) -> TokenStream {
        if self.required.is_empty() {
            return TokenStream::new();
        }

        let params = self.required.iter().map(|r| &r.param);
        quote! { <#(#params),*> }
    }

    /// Same as [`Self::impl_generics`]; type-argument position
    pub fn ty_generics(&self) -> TokenStream {
        self.impl_generics()
    }

    /// Type arguments with `field`'s parameter replaced by its concrete stored type —
    /// the return type of that prop's setters
    pub fn ty_generics_with(&self, field: &Ident) -> TokenStream {
        if self.required.is_empty() {
            return TokenStream::new();
        }

        let args = self.required.iter().map(|r| {
            if &r.field == field {
                r.stored_type.clone()
            } else {
                let param = &r.param;
                quote! { #param }
            }
        });
        quote! { <#(#args),*> }
    }

    /// `<TContent = Stored1, ...>` for the builder struct declaration
    pub fn struct_generics_with_defaults(&self) -> TokenStream {
        if self.required.is_empty() {
            return TokenStream::new();
        }

        let params = self.required.iter().map(|r| {
            let param = &r.param;
            let stored = &r.stored_type;
            quote! { #param = #stored }
        });
        quote! { <#(#params),*> }
    }

    /// `<Missing1, Missing2>` — the state returned by `new()`
    pub fn all_missing_args(&self) -> TokenStream {
        if self.required.is_empty() {
            return TokenStream::new();
        }

        let markers = self.required.iter().map(|r| &r.marker);
        quote! { <#(#markers),*> }
    }

    /// All stored types in required-prop order — the complete builder parameterization
    pub fn all_stored_args(&self) -> TokenStream {
        if self.required.is_empty() {
            return TokenStream::new();
        }

        let stored = self.required.iter().map(|r| &r.stored_type);
        quote! { <#(#stored),*> }
    }
}

pub fn compute_prop_type_ident(prop: &Prop) -> TokenStream {
    let value_type = match &prop.type_ {
        Type::TraitObject(to) => {
            quote! { Box<#to> }
        }
        ty_ => quote! { #ty_ },
    };

    match prop.is_signal {
        Some(SignalType::Vec) => {
            if prop.is_send {
                quote! {futures_signals::signal_vec::BoxSignalVec<'static, #value_type>}
            } else {
                quote! {futures_signals::signal_vec::LocalBoxSignalVec<'static, #value_type>}
            }
        }
        Some(SignalType::Item) => {
            if prop.is_send {
                quote! { futures_signals::signal::BoxSignal<'static, #value_type> }
            } else {
                quote! { futures_signals::signal::LocalBoxSignal<'static, #value_type> }
            }
        }
        _ => quote! { #value_type },
    }
}

/// Human-readable signal type for the generated macro's doc table. This is only ever
/// stringified into docs; `Signal<Item=T> + Send` is not a parseable bare type, so it is
/// built as a string rather than a syn type.
pub fn get_prop_signal_type_param(
    prop: &Prop,
    signal_type: &SignalType,
    prop_type: &Type,
) -> String {
    let send_suffix = if prop.is_send { " + Send" } else { "" };

    match signal_type {
        SignalType::Item => format!(
            "futures_signals::signal::Signal<Item={}>{send_suffix}",
            quote! {#prop_type}
        ),
        SignalType::Vec => format!(
            "futures_signals::signal_vec::SignalVec<Item={}>{send_suffix}",
            quote! {#prop_type}
        ),
    }
}
