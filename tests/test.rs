use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);

#[cfg(test)]
mod test {
    use dominator::Dom;
    use futures_signals::signal::{always, Signal};
    use futures_signals::signal_vec::SignalVecExt;
    use futures_signals::signal_vec::VecDiff;
    use futures_signals_component_macro::component;
    use num_traits::{PrimInt, ToPrimitive};

    #[macro_use]
    pub mod foo {
        use dominator::{html, Dom};
        use futures_signals_component_macro::component;
        use num_traits::PrimInt;

        #[component(render_fn = some_button)]
        pub struct SomeButton<
            FClickCallback: Fn(dominator::events::Click) + Send = fn(
                dominator::events::Click,
            ) -> (),
            T: ToString + Default = i32,
            U: PrimInt + ToString + Default = i32,
        > {
            /// The button label. This can be a signal, which allows us to update the label dynamically based on state changes
            #[signal]
            pub label: String,

            pub click_handler: FClickCallback,

            #[signal]
            #[default("hello".to_string())]
            pub signal_with_default: String,

            #[signal]
            pub foo: T,

            #[signal]
            pub bar: U,

            #[signal_vec]
            #[default(vec ! [123])]
            pub some_generic_signal_vec: i32,

            #[signal]
            #[default({ let foo = 32; foo.to_string() })]
            pub complex_default: String,

            pub unchanging_prop: i32,
        }

        pub fn some_button(props: impl SomeButtonPropsTrait + 'static) -> Dom {
            let SomeButtonProps {
                label,
                signal_with_default,
                click_handler,
                ..
            } = props.take();

            html!("div", {
                .apply_if(label.is_some(), |b| {
                    b.text_signal(label.unwrap())
                })
                .apply_if(click_handler.is_some(), move |b| {
                    let click_handler = click_handler.unwrap();
                    b.event(move |event: dominator::events::Click| {
                        (click_handler)(event);
                    })
                })
                .text_signal(signal_with_default)
            })
        }
    }

    use crate::test::foo::*;

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn cmp_non_macro_test() {
        let _rendered: Dom = some_button(SomeButtonProps::new().foo("hi there").bar(42));
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn cmp_macro_test() {
        let _rendered: Dom = some_button!({
            .foo(42)
            .label("hi there".to_string())
            .unchanging_prop(666)
        });
    }

    // just here to make sure it compiles (it's the example from the readme)
    fn _my_app(label: impl Signal<Item = String> + 'static) -> Dom {
        some_button!({
            .label_signal(label)
            .foo(42)
        })
    }

    #[test]
    fn attr_cmp_test() {
        let t = SomeButtonProps::new();

        let _t = t
            .foo_signal(always("test".to_string()))
            .foo(32)
            .bar(666)
            .label("hi".to_string())
            .label_signal(always("test".to_string()))
            .some_generic_signal_vec_signal_vec(futures_signals::signal_vec::always(vec![42, 666]));
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn default_val_test() {
        #[component(render_fn = _r)]
        struct DefaultVal<T: PrimInt = i32> {
            #[signal]
            #[default(666)]
            foo: i32,

            #[default(123)]
            bar: T,

            #[signal_vec]
            #[default(vec ! [123, 666])]
            baz: i32,
        }

        async fn _r(p: impl DefaultValPropsTrait) {
            let DefaultValProps {
                foo: _, bar, baz, ..
            } = p.take();
            assert_eq!(bar.to_i32().unwrap(), 123);

            let mut vec_val = vec![];

            baz.for_each(|change| {
                if let VecDiff::Replace { values, .. } = change {
                    vec_val = values;
                }

                async {}
            })
            .await;

            assert_eq!(vec_val, vec![123, 666]);
        }

        default_val!({}).await;
    }

    #[test]
    fn verify_send_propagation() {
        let t = trybuild::TestCases::new();

        t.compile_fail("tests/build_fail_checks/nosend.rs");

        #[component(render_fn = render_send)]
        struct NeedsSend<T: Send = (), TNotSend: Clone = ()> {
            #[signal]
            send_me: T,

            #[signal]
            don_not_send_me: TNotSend,
        }

        #[allow(dead_code)]
        fn render_send(props: impl NeedsSendPropsTrait + 'static) -> i32 {
            let NeedsSendProps { send_me, .. } = props.take();

            consume_send(send_me.unwrap());

            42
        }

        #[allow(dead_code)]
        fn consume_send(_: impl Signal<Item = impl Send>) {}
    }
}
