use crate::parse::{Component, Prop, SignalType};
use crate::render::render_utils::{compute_prop_type_ident, BuilderCtx};
use proc_macro2::TokenStream;
use quote::quote;

/// The type a prop is stored as in the props struct: the computed stored type, wrapped in
/// `Option` when the prop is neither defaulted nor required.
fn stored_field_type(prop: &Prop) -> TokenStream {
    let type_ = compute_prop_type_ident(prop);

    if prop.default.is_some() || prop.is_required {
        type_
    } else {
        quote! { Option<#type_> }
    }
}

/// The initializer for a non-required prop in `new()`: the default expression (signal-wrapped
/// as needed) or `None`.
fn default_init_value(prop: &Prop) -> TokenStream {
    if let Some(default) = &prop.default {
        if let Some(sig) = &prop.is_signal {
            match sig {
                SignalType::Item => {
                    if prop.is_send {
                        quote! {futures_signals::signal::always(#default).boxed()}
                    } else {
                        quote! {futures_signals::signal::always(#default).boxed_local()}
                    }
                }
                SignalType::Vec => {
                    if prop.is_send {
                        quote! {futures_signals::signal_vec::always(#default).boxed()}
                    } else {
                        quote! {futures_signals::signal_vec::always(#default).boxed_local()}
                    }
                }
            }
        } else {
            quote! {#default}
        }
    } else {
        quote! {None}
    }
}

pub fn render_prop_builder_struct(ctx: &BuilderCtx, cmp: &Component) -> TokenStream {
    let props_struct_name = &ctx.props_name;

    let props = cmp.props.iter().map(|prop| {
        let name = &prop.name;
        let type_ = stored_field_type(prop);

        quote! {
            pub #name: #type_,
        }
    });

    let docs = cmp.docs.iter().map(|doc| {
        quote! {
            #[doc = #doc]
        }
    });

    let props_struct = quote! {
        #(#docs)*
        pub struct #props_struct_name {
            #(#props)*
        }
    };

    if !ctx.has_required() {
        return render_plain_ctor(ctx, cmp, props_struct);
    }

    render_builder(ctx, cmp, props_struct)
}

/// No required props: `new()` on the props struct itself, exactly the 0.4 shape, plus a no-op
/// `build()` so callers can treat every component's props chain uniformly.
fn render_plain_ctor(ctx: &BuilderCtx, cmp: &Component, props_struct: TokenStream) -> TokenStream {
    let props_struct_name = &ctx.props_name;

    let props_ctor = cmp.props.iter().map(|prop| {
        let name = &prop.name;
        let init_val = default_init_value(prop);

        quote! {
            #name: #init_val,
        }
    });

    // A 0.4-era component may legitimately have a prop named `build`; its setter keeps
    // precedence and we skip the no-op finalizer to stay backwards compatible.
    let build_fn = if cmp.props.iter().any(|p| p.name == "build") {
        TokenStream::new()
    } else {
        quote! {
            pub fn build(self) -> Self {
                self
            }
        }
    };

    quote! {
        #props_struct

        impl #props_struct_name {
            pub fn new() -> Self {
                use futures_signals::signal::SignalExt;
                use futures_signals::signal_vec::SignalVecExt;

                Self {
                    #(#props_ctor)*
                }
            }

            #build_fn
        }
    }
}

/// Required props present: generate the typestate builder, markers, and the completion
/// trait carrying the missing-prop diagnostic.
fn render_builder(ctx: &BuilderCtx, cmp: &Component, props_struct: TokenStream) -> TokenStream {
    let props_struct_name = &ctx.props_name;
    let builder_name = &ctx.builder_name;
    let complete_trait = &ctx.complete_trait;
    let render_fn = &cmp.render_fn;

    let markers = ctx.required.iter().map(|r| {
        let marker = &r.marker;
        let field = r.field.to_string();
        let doc = format!(
            "Typestate marker: the required `{field}` prop of [`{props_struct_name}`] has not been set yet."
        );

        quote! {
            #[doc = #doc]
            pub struct #marker;
        }
    });

    let builder_fields = cmp.props.iter().map(|prop| {
        let name = &prop.name;

        let type_ = if let Some(required) = ctx.required.iter().find(|r| &r.field == name) {
            let param = &required.param;
            quote! { #param }
        } else {
            stored_field_type(prop)
        };

        quote! {
            #name: #type_,
        }
    });

    let builder_ctor_fields = cmp.props.iter().map(|prop| {
        let name = &prop.name;

        let init_val = if let Some(required) = ctx.required.iter().find(|r| &r.field == name) {
            let marker = &required.marker;
            quote! { #marker }
        } else {
            default_init_value(prop)
        };

        quote! {
            #name: #init_val,
        }
    });

    let build_props_fields = cmp.props.iter().map(|prop| {
        let name = &prop.name;

        quote! {
            #name: self.#name,
        }
    });

    let struct_generics = ctx.struct_generics_with_defaults();
    let impl_generics = ctx.impl_generics();
    let ty_generics = ctx.ty_generics();
    let all_missing = ctx.all_missing_args();
    let all_stored = ctx.all_stored_args();

    let required_names = ctx
        .required
        .iter()
        .map(|r| format!("`{}`", r.field))
        .collect::<Vec<_>>()
        .join(", ");
    let required_setters = ctx
        .required
        .iter()
        .map(|r| format!("`.{}(...)`", r.field))
        .collect::<Vec<_>>()
        .join(", ");

    let builder_doc = format!(
        "Typestate builder for [`{props_struct_name}`]. \
         Required props: {required_names}. \
         `.build()` becomes available once every required prop has been set."
    );
    let on_unimplemented_message = format!("missing required props for component `{render_fn}`");
    let on_unimplemented_label = format!(
        "required props: {required_names} — set each with {required_setters} before calling `.build()` (the `{render_fn}` component macro calls it for you)"
    );
    let on_unimplemented_note = format!(
        "the builder state is `{{Self}}`; each `{builder_name}Missing*` type parameter is a prop that has not been set"
    );

    quote! {
        #props_struct

        #(#markers)*

        #[doc = #builder_doc]
        pub struct #builder_name #struct_generics {
            #(#builder_fields)*
        }

        impl #props_struct_name {
            pub fn new() -> #builder_name #all_missing {
                use futures_signals::signal::SignalExt;
                use futures_signals::signal_vec::SignalVecExt;

                #builder_name {
                    #(#builder_ctor_fields)*
                }
            }
        }

        #[diagnostic::on_unimplemented(
            message = #on_unimplemented_message,
            label = #on_unimplemented_label,
            note = #on_unimplemented_note
        )]
        pub trait #complete_trait {
            fn build_props(self) -> #props_struct_name;
        }

        impl #complete_trait for #builder_name #all_stored {
            fn build_props(self) -> #props_struct_name {
                #props_struct_name {
                    #(#build_props_fields)*
                }
            }
        }

        impl #impl_generics #builder_name #ty_generics {
            pub fn build(self) -> #props_struct_name
            where
                Self: #complete_trait,
            {
                #complete_trait::build_props(self)
            }
        }
    }
}
