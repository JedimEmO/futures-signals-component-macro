# futures-signals-component-macros

This crates provides utility macros for making components based on `futures-signals`.
Its purpose is to generate macro-style components that are flexible to use with both signal and non-signal properties, while not overloading the component implementation with type complexity.

Here's an example of how to create a component (in this case the output is a [DOMINATOR](https://github.com/Pauan/rust-dominator) `Dom` node, but it can be any rust type:

```rust
#[component(render_fn = some_button)]
pub struct SomeButton<T: ToString + Default = i32, U: ToI32 + ToString + Default = i32> {
    /// The button label. This can be a signal, which allows us to update the label dynamically based on state changes
    /// The macro also generates a setter for a non-signal setter, in case we just want to assign a static value to the property
    #[signal]
    pub label: String,
    
    #[signal]
    pub foo: T,

    #[signal]
    pub bar: U,

    #[signal_vec]
    #[default(vec![123])]
    pub some_generic_signal_vec: i32,
}

pub fn some_button(props: impl SomeButtonPropsTrait + 'static) -> Dom {
    let SomeButtonProps { label, .. } = props.take();

    html!("div", {
        .apply_if(label.is_some(), |b| {
            b.text_signal(label.unwrap())
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