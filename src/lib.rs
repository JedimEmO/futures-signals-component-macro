mod parse;
mod render;

use crate::parse::parse_field::parse_field;
use crate::parse::AttributeArgument;
use crate::parse::Component;
use crate::render::render_props;
use proc_macro::TokenStream;
use syn::Meta;

/// This attribute macro is meant to simplify making components using `futures-signals` for their properties.
/// It lets you declare your components inputs in form of a normal, attribute annotated rust struct.
///
/// It generates a component macro and a props struct with a chainable setter API for the annotated struct.
///
/// ## Field attributes
/// The fields of the component struct can be annotated with the following attributes:
///
/// ### `#[signal]`
/// Fields annotated with this attribute will have two setter functions created: `field_name()` and `field_name_signal()`.
///
/// ### `#[signal_vec]`
/// This behaves much like the `#[signal]` attribute, but will make the field a `SignalVec` rather than a `Signal`
///
/// ### `#[default({expr})]`
/// Lets you chose a default value for the field, in terms of an expression.
/// This means you can use both literals and more complex blocks to choose a default value:
///
/// Fields that are not annotated with the `#[default]` or `#[required]` attributes are optional,
/// and their type is wrapped in the `Option` enum.
///
/// ```
/// # use futures_signals_component_macro::component;
/// #[component(render_fn = my_cmp)]
/// struct MyCmp {
///     #[default(42)]
///     my_int: i32,
///     #[default({ let foo = 32; foo.to_string() })]
///     my_string: String
/// }
/// # fn my_cmp(_props: MyCmpProps) {}
/// ```
///
/// ### `#[required]`
/// Marks the field as mandatory: the component will not compile unless the caller provides a
/// value for it, through either of its setters. Enforcement happens at compile time via a
/// typestate builder — `MyCmpProps::new()` returns a `MyCmpPropsBuilder` whose `.build()`
/// method only exists once every required prop has been set. The props struct itself stays a
/// plain struct, and required fields are stored unwrapped (no `Option`).
///
/// ```
/// # use futures_signals_component_macro::component;
/// #[component(render_fn = user_badge)]
/// struct UserBadge {
///     #[signal]
///     #[required]
///     username: String,
///
///     #[signal]
///     #[default("guest".to_string())]
///     role: String,
/// }
///
/// fn user_badge(props: UserBadgeProps) {
///     let UserBadgeProps { username, role, .. } = props;
///     // `username` is a plain LocalBoxSignal<'static, String> here — no Option, no unwrap
///     let _ = (username, role);
/// }
///
/// # fn main() {
/// user_badge!({ .username("ferris".to_string()) });
/// # }
/// ```
///
/// Omitting a required prop is a compile error:
///
/// ```compile_fail
/// # use futures_signals_component_macro::component;
/// # #[component(render_fn = user_badge)]
/// # struct UserBadge {
/// #     #[signal]
/// #     #[required]
/// #     username: String,
/// # }
/// # fn user_badge(props: UserBadgeProps) { let _ = props.username; }
/// # fn main() {
/// user_badge!({ }); // error: missing required props for component `user_badge`
/// # }
/// ```
///
/// ### `#[into]`
/// Relaxes the generated setters to accept any type convertible into the prop type: the value
/// setter takes `impl Into<T>`, and the `_signal` setter accepts signals of any item type
/// implementing `Into<T>` (items are converted as they arrive). Handy for `String` props
/// (pass `&str` literals) and `Option<T>` props (pass a bare `T`).
///
/// ```
/// # use futures_signals_component_macro::component;
/// #[component(render_fn = greeting)]
/// struct Greeting {
///     #[signal]
///     #[into]
///     #[default(String::new())]
///     message: String,
/// }
/// # fn greeting(_props: GreetingProps) {}
/// # fn main() {
/// greeting!({ .message("hello") });
/// greeting!({ .message_signal(futures_signals::signal::always("hi")) });
/// # }
/// ```
///
/// ## The `render_fn`
///
/// Your `render_fn` receives the generated `MyComponentProps` struct by value and can destructure
/// it directly. Fields hold the stored prop types: `#[signal]` props are boxed signals
/// (`LocalBoxSignal<'static, T>`, or `BoxSignal` with `#[send]`), `#[signal_vec]` props are boxed
/// signal vecs, trait-object props are boxed, and optional props (no `#[default]`, no
/// `#[required]`) are wrapped in `Option`.
///
/// ```rust
/// # use futures_signals_component_macro::component;
/// #[component(render_fn=take_destruct_example)]
/// struct TakeDestructExample {
///     #[signal]
///     optional_string_signal: String,
///
///     #[signal]
///     #[default("hi".to_string())]
///     string_signal: String,
/// }
///
/// fn take_destruct_example(props: TakeDestructExampleProps) {
///     let TakeDestructExampleProps {
///         optional_string_signal /* this has the type Option<LocalBoxSignal<'static, String>> */,
///         string_signal /* this has the type LocalBoxSignal<'static, String> */
/// # , ..
/// } = props;
/// }
/// ```
///
/// The return type of your render_fn should be the component type your rendering library expects.
/// In the examples we use the DOMINATOR dom node, but you can use the `#[component]` macro to produce components for any library working with `futures-signals`.
///
/// ## The `apply` prop (dominator feature)
///
/// With the `dominator` feature enabled, every component gets an `apply` prop accepting a
/// `FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement>` for ad-hoc customization of the
/// component's root element. Calling `.apply(...)` several times composes the functions —
/// the first one applied runs first. The field name `apply` is therefore reserved.
///
/// # Example:
///
/// Here's a full component example, making a clickable button using the DOMINATOR `html!` macro.
/// Notice the use of the `some_button!` macro in the `my_app()` function.
///
/// ```rust
/// # use dominator::{Dom, html};
/// # use futures_signals::signal::Signal;
/// # use futures_signals_component_macro::component;
/// # use num_traits::PrimInt;
/// #[component(render_fn = some_button)]
/// pub struct SomeButton {
///     #[signal]
///     pub label: String,
///
///     pub click_handler: dyn Fn(dominator::events::Click) -> () + Send + 'static,
///
///     pub boxed_click_handler: Box<dyn Fn(dominator::events::Click) -> ()>,
///
///     #[signal]
///     pub foo: dyn ToString + Send + 'static,
///     #[signal_vec]
///     #[default(vec ! [123])]
///     pub some_generic_signal_vec: i32,
/// }
///
/// pub fn some_button(props: SomeButtonProps) -> Dom {
///     let SomeButtonProps { label, click_handler, boxed_click_handler, .. } = props;
///
///     html!("div", {
///         .apply_if(click_handler.is_some(), move |b| {
///             let click_handler = click_handler.unwrap();
///             b.event(move |event: dominator::events::Click| {
///                 (click_handler)(event);
///             })
///         })
///         .apply_if(boxed_click_handler.is_some(), move |b| {
///             let boxed_click_handler = boxed_click_handler.unwrap();
///             b.event(move |event: dominator::events::Click| {
///                 (boxed_click_handler)(event);
///             })
///         })
///         .apply_if(label.is_some(), |b| {
///             b.text_signal(label.unwrap())
///         })
///     })
/// }
///
/// // Usage
/// fn my_app(label: impl Signal<Item=String> + Send + 'static) -> Dom {
///     some_button!({
///         .label_signal(label)
///         .foo(42)
///     })
/// }
/// ```
#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    match component_impl(args, input) {
        Ok(out) => out,
        Err(e) => e.to_compile_error().into(),
    }
}

fn component_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let struct_ = syn::parse::<syn::ItemStruct>(input)?;
    let arg = syn::parse::<AttributeArgument>(args).map_err(|e| {
        syn::Error::new(
            e.span(),
            "expected `#[component(render_fn = your_render_fn)]`",
        )
    })?;

    let docs = struct_
        .attrs
        .into_iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(docstring) = attr.meta {
                    Some(docstring.value)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let fields = match struct_.fields {
        syn::Fields::Named(fields) => fields.named,
        _ => {
            return Err(syn::Error::new(
                struct_.ident.span(),
                "component struct must have named fields",
            ))
        }
    };

    let props = fields
        .iter()
        .map(parse_field)
        .collect::<syn::Result<syn::punctuated::Punctuated<parse::Prop, syn::Token![,]>>>()?;

    #[cfg(feature = "dominator")]
    if let Some(prop) = props.iter().find(|p| p.name == "apply") {
        return Err(syn::Error::new(
            prop.name.span(),
            "the field name `apply` is reserved; the component macro injects an `apply` prop when the `dominator` feature is enabled",
        ));
    }

    if props.iter().any(|p| p.is_required) {
        if let Some(prop) = props.iter().find(|p| p.name == "build") {
            return Err(syn::Error::new(
                prop.name.span(),
                "the field name `build` is reserved on components with `#[required]` props; it is used by the builder finalizer",
            ));
        }
    }

    #[cfg(feature = "dominator")]
    let mut cmp: Component = Component {
        name: struct_.ident,
        render_fn: arg.fn_name,
        props,
        docs,
    };

    #[cfg(not(feature = "dominator"))]
    let cmp: Component = Component {
        name: struct_.ident,
        render_fn: arg.fn_name,
        props,
        docs,
    };

    #[cfg(feature = "dominator")]
    {
        let apply_prop = parse::Prop {
            is_signal: None,
            is_send: false,
            is_required: false,
            is_into: false,
            is_compose_apply: true,
            name: syn::Ident::new("apply", cmp.name.span()),
            type_: syn::parse_str::<syn::Type>("dyn FnOnce(dominator::DomBuilder<web_sys::HtmlElement>) -> dominator::DomBuilder<web_sys::HtmlElement> + 'static").expect("failed to parse type"),
            default: None,
            docs: vec![],
        };

        cmp.props.push(apply_prop);
    }

    Ok(render_props(&cmp).into())
}
