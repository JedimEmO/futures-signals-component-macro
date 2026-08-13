# futures-signals-component-macros

This crates provides utility macros for making components based on `futures-signals`.
Its purpose is to generate macro-style components that are flexible to use with both signal and non-signal properties, while not overloading the component implementation with type complexity.

Here's an example of how to create a component (in this case the output is a [DOMINATOR](https://github.com/Pauan/rust-dominator) `Dom` node, but it can be any rust type:

```rust
#[component(render_fn = some_button)]
pub struct SomeButton {
    /// The button label. This can be a signal, which allows us to update the label dynamically based on state changes.
    /// The macro also generates a non-signal setter, in case we just want to assign a static value to the property.
    /// `#[required]` makes omitting the label a compile error; `#[into]` lets setters accept `&str`.
    #[signal]
    #[required]
    #[into]
    pub label: String,

    #[signal]
    pub foo: dyn ToString + 'static,

    #[signal_vec]
    #[default(vec![123])]
    pub some_generic_signal_vec: i32,
}

pub fn some_button(props: SomeButtonProps) -> Dom {
    let SomeButtonProps { label, foo, .. } = props;

    html!("div", {
        // `label` is required, so it arrives as a plain boxed signal — no Option, no unwrap
        .text_signal(label)
        // `foo` is optional (no #[default], no #[required]), so it is an Option
        .apply_if(foo.is_some(), |b| {
            b.text_signal(foo.unwrap().map(|v| v.to_string()))
        })
    })
}
```

To use this component, you can then use the generated `some_button!` macro, like so:

```rust
fn my_app(label: impl Signal<Item=String> + 'static) -> Dom {
    some_button!({
        .label_signal(label)
        .foo(42)
    })
}
```

Omitting a `#[required]` prop fails to compile with a targeted error, both through the macro and
through the direct builder chain (`SomeButtonProps::new().foo(42).build()`).

Field attributes:

| Attribute | Effect |
|---|---|
| `#[signal]` | Generates `name()` and `name_signal()` setters; stored as a boxed signal |
| `#[signal_vec]` | Like `#[signal]`, but for `SignalVec` |
| `#[default(expr)]` | Seeds the prop with `expr`; the prop is stored unwrapped |
| `#[required]` | The caller must set the prop — enforced at compile time via a typestate builder |
| `#[into]` | Setters accept `impl Into<T>` (and signals of `Into<T>` items) |
| `#[send]` | Boxed signal types are `Send` (`BoxSignal` instead of `LocalBoxSignal`) |

Props with none of `#[default]`/`#[required]` are optional and stored as `Option<T>`.

## Developing and testing

To run the tests locally, you need a few dependencies on your system.

First of all, you need rust.
Install it following the instructions for your system at https://rustup.rs/

You also need the `wasm32-unknown-unknown` target:

```shell
rustup target add wasm32-unknown-unknown
```

And finally you will need the `wasm-bindgen-cli` tool to be able to run the in-browser tests:

```shell
cargo install wasm-bindgen-cli
```

Now you can run tests with the following commands:

```shell
cargo test & 
cargo test --target wasm32-unknown-unknown
```