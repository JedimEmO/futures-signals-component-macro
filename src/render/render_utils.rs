use crate::parse::{Prop, SignalType};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Type, TypeParam};

pub fn new_prop_signal_name(prop_name: &Ident) -> String {
    format!("T{}SignalNew", prop_name)
}

pub fn prop_signal_name(prop_name: &Ident) -> String {
    format!("T{}Signal", prop_name)
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
            quote! {futures_signals::signal_vec::BoxSignalVec<'static, #value_type>}
        }
        Some(SignalType::Item) => {
            quote! {futures_signals::signal::BoxSignal<'static, #value_type>}
        }
        _ => quote! { #value_type },
    }
}

pub fn get_prop_signal_type_param(
    prop: &Prop,
    signal_type: &SignalType,
    prop_type: &Type,
    is_new: bool,
) -> TypeParam {
    let signal_name = if is_new {
        new_prop_signal_name(&prop.name)
    } else {
        prop_signal_name(&prop.name)
    };

    let is_send = prop.is_send;

    let send_suffix = if is_send { " + Send" } else { "" };

    match signal_type {
        SignalType::Item => syn::parse_str(
            format!(
                "{}: futures_signals::signal::Signal<Item={}> {send_suffix}",
                signal_name,
                quote! {#prop_type}
            )
            .as_str(),
        )
        .expect("failed to parse signal generic"),

        SignalType::Vec => syn::parse_str(
            format!(
                "{}: futures_signals::signal_vec::SignalVec<Item={}> {send_suffix}",
                signal_name,
                quote! {#prop_type}
            )
            .as_str(),
        )
        .expect("failed to parse signal generic"),
    }
}
