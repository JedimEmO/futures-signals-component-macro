mod parse;
mod render;

use crate::parse::parse_field::parse_field;
use crate::parse::AttributeArgument;
use crate::parse::{Component, PropGenerics};
use crate::render::render_props;
use proc_macro::TokenStream;
use syn::{GenericArgument, Meta, PathArguments, Type};

/// This attribute macro is meant to simplify making components using `futures-signals` for their properties.
/// It lets you declare your components inputs in form of a normal, attribute annotated rust struct.
///
/// It generates a component macro, a props struct and the prop structs builder + trait for the annotated struct.
///
/// ## Field attributes
/// The fields of the component struct can be annotated with the following attributes:
///
/// ### `#[signal]`
/// Fields annotated with this attribute will have to setter functions created on the builder: `field_name()` and `field_name_signal()`.
///
/// ### `#[signal_vec]`
/// This behaves much like the `#[signal]` attribute, but will make the field a `SignalVec` rather than a `Signal`
///
/// ### `#[default({expr})]`
/// Lets you chose a default value for the field, in terms of an expression.
/// This means you can use both literals and more complex blocks to choose a default value:
///
/// Fields that are not annotated with the `#[default]` attribute are optional, and their type is wrapped in the `Option` enum.
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
/// ```
///
/// ## The `render_fn`
///
/// Your `render_fn` can not know the concrete type of your props struct, as it is heavily generic, and changes based on properties the user of the builder chose.
///
/// It should always accept and argument that implements the generated `MyComponentPropsTrait` trait.
/// This impl in turn has a `take()` method, which will consume the props struct and output Self, which will give us access to its member values (these match our fields in the original struct by identifier).
///
/// We can then destruct the Self into variables:
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
/// fn take_destruct_example(props: impl TakeDestructExamplePropsTrait + 'static) {
///     let TakeDestructExampleProps {
///         optional_string_signal /* this has the type Option<impl Signal<Item=String>> */,
///         string_signal /* this has the type impl Signal<Item=String> */
/// # , ..
/// } = props.take();
/// }
/// ```
///
/// The return type of your render_fn should be the component type your rendering library expects.
/// In the examples we use the DOMINATOR dom node, but you can use the `#[component]` macro to produce components for any library working with `futures-signals`.
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
/// pub struct SomeButton<FClickCallback: Fn(dominator::events::Click) -> () = fn(dominator::events::Click) -> (), T: ToString + Default = i32, U: PrimInt + Default = i32> {
///     #[signal]
///     pub label: String,
///
///     pub click_handler: FClickCallback,
///
///     pub boxed_click_handler: Box<dyn Fn(dominator::events::Click) -> ()>,
///
///     #[signal]
///     pub foo: T,
///
///     #[signal]
///     pub bar: U,
///
///     #[signal_vec]
///     #[default(vec ! [123])]
///     pub some_generic_signal_vec: i32,
/// }
///
/// pub fn some_button(props: impl SomeButtonPropsTrait + 'static) -> Dom {
///     let SomeButtonProps { label, click_handler, boxed_click_handler, .. } = props.take();
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
/// fn my_app(label: impl Signal<Item=String> + 'static) -> Dom {
///     some_button!({
///         .label_signal(label)
///         .foo(42)
///     })
/// }
/// ```
#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    let struct_ = syn::parse::<syn::ItemStruct>(input).expect("failed to parse struct");
    let arg = syn::parse::<AttributeArgument>(args).expect("failed to parse attribute args");

    let docs = struct_
        .attrs
        .into_iter()
        .filter_map(|attr| {
            if *attr.path().get_ident().unwrap() == "doc" {
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
        _ => panic!("struct must have named fields"),
    };

    let struct_generics = struct_
        .generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Type(type_param) => PropGenerics {
                param: type_param.clone(),
            },
            _ => panic!("prop struct must have only type params"),
        })
        .collect::<Vec<_>>();

    let fields = fields
        .iter()
        .map(|field| parse_field(field, &struct_generics));

    let cmp: Component = Component {
        name: struct_.ident,
        render_fn: arg.fn_name,
        props: fields.collect(),
        docs,
    };

    #[cfg(feature = "dominator")]
    let apply_prop = parse::Prop {
        is_signal: None,
        name: syn::Ident::new("apply", cmp.name.span()),
        generics: Some(PropGenerics { param: syn::parse_str::<syn::TypeParam>("TApplyFn: FnOnce(dominator::DomBuilder<web_sys::HtmlElement>) -> dominator::DomBuilder<web_sys::HtmlElement> = fn(dominator::DomBuilder<web_sys::HtmlElement>)->dominator::DomBuilder<web_sys::HtmlElement>").expect("failed to parse type param") }),
        type_: syn::parse_str::<Type>("TApplyFn").expect("failed to parse type"),
        default: None,
        docs: vec![],
    };

    #[cfg(feature = "dominator")]
    cmp.props.push(apply_prop);

    render_props(&cmp).into()
}

fn get_type_generic_param_use(
    type_: &Type,
    struct_generics: &Vec<PropGenerics>,
) -> Vec<PropGenerics> {
    let mut out = vec![];

    if let Type::Path(type_path) = &type_ {
        for segment in &type_path.path.segments {
            if let Some(generic) = struct_generics
                .iter()
                .find(|generic| segment.ident == generic.param.ident)
            {
                out.push(generic.clone());
            }

            if let PathArguments::AngleBracketed(angle_bracketed_arguments) = &segment.arguments {
                for argument in &angle_bracketed_arguments.args {
                    match &argument {
                        GenericArgument::Type(Type::Path(generic_type)) => {
                            for segment in generic_type.path.segments.iter() {
                                if let Some(generic) = struct_generics
                                    .iter()
                                    .find(|generic| segment.ident == generic.param.ident)
                                {
                                    out.push(generic.clone());
                                }
                            }
                        }
                        GenericArgument::Type(type_) => {
                            out.append(&mut get_type_generic_param_use(type_, struct_generics));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    out
}
