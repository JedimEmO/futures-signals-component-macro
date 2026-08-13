pub mod render_component_macro;
pub mod render_prop_impl;
pub mod render_props_builder_struct;
pub mod render_utils;

use crate::parse::Component;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

use crate::render::render_component_macro::render_component_macro;

use crate::render::render_prop_impl::render_prop_impl;
use crate::render::render_props_builder_struct::render_prop_builder_struct;
use crate::render::render_utils::BuilderCtx;

/// Renders the props builder struct along with all the impls of type changing prop setters
pub fn render_props(cmp: &Component) -> TokenStream {
    let ctx = BuilderCtx::new(cmp);

    let props_struct_ts = render_prop_builder_struct(&ctx, cmp);
    let props_impl_ts = cmp
        .props
        .iter()
        .map(|prop| render_prop_impl(&ctx, prop, cmp));
    let macro_ = render_component_macro(&ctx, cmp);

    let mut s = quote! {
        #props_struct_ts
        #(#props_impl_ts)*
    };

    s.extend(macro_);
    s
}

fn render_doc_exprs(doc_exprs: &Vec<Expr>) -> TokenStream {
    let mut s = TokenStream::new();

    for doc_expr in doc_exprs {
        s.extend(quote! {
            #[doc = #doc_expr]
        });
    }

    s
}
