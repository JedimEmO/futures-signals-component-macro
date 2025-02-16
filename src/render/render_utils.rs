use crate::parse::{Prop, SignalType};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Type, TypePath};

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

pub fn get_prop_signal_type_param(
    prop: &Prop,
    signal_type: &SignalType,
    prop_type: &Type,
) -> TypePath {
    let is_send = prop.is_send;
    let send_suffix = if is_send { " + Send" } else { "" };

    match signal_type {
        SignalType::Item => syn::parse_str(
            format!(
                "futures_signals::signal::Signal<Item={}> {send_suffix}",
                quote! {#prop_type}
            )
            .as_str(),
        )
        .expect("failed to parse signal generic"),

        SignalType::Vec => syn::parse_str(
            format!(
                "futures_signals::signal_vec::SignalVec<Item={}> {send_suffix}",
                quote! {#prop_type}
            )
            .as_str(),
        )
        .expect("failed to parse signal generic"),
    }
}
