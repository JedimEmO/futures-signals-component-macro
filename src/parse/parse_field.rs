use crate::parse::{docs_from_attrs, Prop, SignalType};
use syn::spanned::Spanned;
use syn::{Field, Type};

pub fn parse_field(field: &Field) -> syn::Result<Prop> {
    let is_signal = field.attrs.iter().any(|a| a.path().is_ident("signal"));
    let is_signal_vec = field.attrs.iter().any(|a| a.path().is_ident("signal_vec"));
    let is_send = field.attrs.iter().any(|a| a.path().is_ident("send"));
    let is_required = field.attrs.iter().any(|a| a.path().is_ident("required"));
    let is_into = field.attrs.iter().any(|a| a.path().is_ident("into"));

    let default = field
        .attrs
        .iter()
        .find(|a| a.path().is_ident("default"))
        .map(|a| a.parse_args::<syn::Expr>())
        .transpose()?;

    if is_signal && is_signal_vec {
        return Err(syn::Error::new(
            field.span(),
            "field cannot be both signal and signal_vec",
        ));
    }

    if is_required && default.is_some() {
        return Err(syn::Error::new(
            field.span(),
            "`#[required]` and `#[default(...)]` are contradictory; a required prop must be provided by the caller",
        ));
    }

    if is_into && is_signal_vec {
        return Err(syn::Error::new(
            field.span(),
            "`#[into]` is not supported on `#[signal_vec]` props; the value setter already accepts `impl Into<Vec<T>>`",
        ));
    }

    if is_into && matches!(field.ty, Type::TraitObject(_)) {
        return Err(syn::Error::new(
            field.span(),
            "`#[into]` cannot be used on trait-object props; the setter already accepts `impl Trait`",
        ));
    }

    let field_docs = docs_from_attrs(field.attrs.iter());

    Ok(Prop {
        is_signal: if is_signal {
            Some(SignalType::Item)
        } else if is_signal_vec {
            Some(SignalType::Vec)
        } else {
            None
        },
        is_send,
        is_required,
        is_into,
        is_compose_apply: false,
        name: field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new(field.span(), "field must have a name"))?,
        type_: field.ty.clone(),
        default,
        docs: field_docs,
    })
}
