use crate::parse::{Component, SignalType};
use crate::render::render_utils::compute_prop_type_ident;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn render_prop_builder_struct(props_struct_name: Ident, cmp: &Component) -> TokenStream {
    let props = cmp.props.iter().map(|prop| {
        let name = &prop.name;
        let type_ = compute_prop_type_ident(prop);

        let type_ = if let Some(_default) = &prop.default {
            type_
        } else {
            quote! { Option<#type_> }
        };

        quote! {
            pub #name: #type_,
        }
    });

    let props_ctor = cmp.props.iter().map(|prop| {
        let name = &prop.name;

        let init_val = if prop.default.is_some() {
            let default = prop.default.as_ref().unwrap();

            if let Some(sig) = &prop.is_signal {
                match sig {
                    SignalType::Item => quote! {futures_signals::signal::always(#default).boxed()},
                    SignalType::Vec => quote! {futures_signals::signal_vec::always(#default).boxed()},
                }
            } else {
                quote! {#default}
            }
        } else {
            quote! {None}
        };

        quote! {
            #name: #init_val,
        }
    });

    let docs = cmp.docs.iter().map(|doc| {
        quote! {
            #[doc = #doc]
        }
    });

    quote! {
        #(#docs)*
        pub struct #props_struct_name {
            #(#props)*
        }


        impl #props_struct_name {
            pub fn new() -> Self {
                use futures_signals::signal::SignalExt;
                use futures_signals::signal_vec::SignalVecExt;

                Self {
                    #(#props_ctor)*
                }
            }
        }
    }
}
