use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);

#[cfg(test)]
mod test {
    use dominator::Dom;
    use futures_signals::signal::{always, Signal};
    use futures_signals::signal_vec::SignalVecExt;
    use futures_signals::signal_vec::VecDiff;
    use futures_signals_component_macro::component;

    #[macro_use]
    pub mod foo {
        use dominator::{html, Dom};
        use futures_signals_component_macro::component;

        #[component(render_fn = some_button)]
        pub struct SomeButton {
            /// The button label. This can be a signal, which allows us to update the label dynamically based on state changes
            #[signal]
            pub label: String,

            pub click_handler: dyn Fn(dominator::events::Click) + 'static,

            #[signal]
            #[default("hello".to_string())]
            pub signal_with_default: String,

            #[signal]
            pub foo: dyn ToString + 'static,

            #[signal_vec]
            #[default(vec ! [123])]
            pub some_generic_signal_vec: i32,

            #[signal]
            #[default({ let foo = 32; foo.to_string() })]
            pub complex_default: String,

            pub unchanging_prop: i32,
        }

        pub fn some_button(props: SomeButtonProps) -> Dom {
            let SomeButtonProps {
                label,
                signal_with_default,
                click_handler,
                ..
            } = props;

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
        let _rendered: Dom = some_button(SomeButtonProps::new().foo("hi there"));
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
    fn _my_app(label: impl Signal<Item = String> + Send + 'static) -> Dom {
        some_button!({
            .label_signal(label)
            .foo(42)
        })
    }

    #[test]
    fn attr_cmp_test() {
        let t = SomeButtonProps::new();

        let foo: Box<dyn ToString + 'static> = Box::new("test".to_string());

        let _t = t
            .foo_signal(always(foo))
            .foo(32)
            .label("hi".to_string())
            .label_signal(always("test".to_string()))
            .some_generic_signal_vec_signal_vec(futures_signals::signal_vec::always(vec![42, 666]));
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn default_val_test() {
        #[component(render_fn = _r)]
        struct DefaultVal {
            #[signal]
            #[default(666)]
            foo: i32,

            #[default(Box::new("123"))]
            bar: dyn ToString + 'static,

            #[signal_vec]
            #[default(vec ! [123, 666])]
            baz: i32,
        }

        async fn _r(p: DefaultValProps) {
            let DefaultValProps {
                foo: _, bar, baz, ..
            } = p;
            assert_eq!(bar.to_string(), "123");

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

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn required_props_test() {
        #[component(render_fn = required_cmp)]
        struct RequiredCmp {
            #[signal]
            #[required]
            title: String,

            #[required]
            formatter: dyn ToString + 'static,

            #[signal]
            #[default(0)]
            count: i32,
        }

        fn required_cmp(props: RequiredCmpProps) -> String {
            let RequiredCmpProps { formatter, .. } = props;
            formatter.to_string()
        }

        // Required props can be set in any order, via either setter flavor, through the
        // direct builder chain or the generated macro.
        let out = required_cmp(
            RequiredCmpProps::new()
                .formatter(42)
                .title("t".to_string())
                .build(),
        );
        assert_eq!(out, "42");

        let out = required_cmp(
            RequiredCmpProps::new()
                .count(3)
                .title_signal(always("t".to_string()))
                .formatter("str")
                .build(),
        );
        assert_eq!(out, "str");

        let out = required_cmp!({
            .formatter(7)
            .title("hi".to_string())
        });
        assert_eq!(out, "7");
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn required_send_signal_test() {
        #[component(render_fn = send_cmp)]
        struct SendCmp {
            #[signal]
            #[send]
            #[required]
            label: String,
        }

        fn send_cmp(props: SendCmpProps) -> impl Signal<Item = String> + Send {
            props.label
        }

        fn assert_send<T: Send>(_: &T) {}

        let sig = send_cmp(SendCmpProps::new().label("x".to_string()).build());
        assert_send(&sig);
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn into_props_test() {
        #[component(render_fn = into_cmp)]
        struct IntoCmp {
            #[signal]
            #[into]
            #[default(String::new())]
            message: String,

            #[into]
            #[default(None)]
            opt_label: Option<String>,

            #[into]
            #[required]
            id: String,
        }

        fn into_cmp(props: IntoCmpProps) -> (Option<String>, String) {
            let IntoCmpProps { opt_label, id, .. } = props;
            (opt_label, id)
        }

        // Value setters accept impl Into<T>: &str for a String prop, a bare T for an
        // Option<T> prop. The _signal setter converts items as they arrive.
        let (opt, id) = into_cmp(
            IntoCmpProps::new()
                .message("hello")
                .message_signal(always("hi"))
                .opt_label("maybe".to_string())
                .id("my-id")
                .build(),
        );
        assert_eq!(opt, Some("maybe".to_string()));
        assert_eq!(id, "my-id");

        let (opt, _) = into_cmp!({ .id("x") });
        assert_eq!(opt, None);
    }

    #[cfg(feature = "dominator")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn apply_compose_test() {
        use dominator::html;

        #[component(render_fn = apply_cmp)]
        struct ApplyCmp {}

        fn apply_cmp(props: ApplyCmpProps) -> Dom {
            let ApplyCmpProps { apply } = props;

            html!("div", {
                .apply_if(apply.is_some(), |b| b.apply(apply.unwrap()))
            })
        }

        // The no-op build() keeps direct builder chains uniform on components without
        // required props.
        let _no_apply: Dom = apply_cmp(ApplyCmpProps::new().build());

        let dom = apply_cmp!({
            .apply(|b| b.attr("data-first", "1").attr("data-winner", "first"))
            .apply(|b| b.attr("data-second", "2").attr("data-winner", "second"))
        });

        dominator::append_dom(&dominator::body(), dom);

        let document = web_sys::window().unwrap().document().unwrap();
        let el = document.query_selector("[data-first]").unwrap().unwrap();

        // Both apply fns ran (composition instead of the 0.4 overwrite), ...
        assert_eq!(el.get_attribute("data-first").as_deref(), Some("1"));
        assert_eq!(el.get_attribute("data-second").as_deref(), Some("2"));
        // ... in call order: the later .apply() runs last and wins conflicting writes.
        assert_eq!(el.get_attribute("data-winner").as_deref(), Some("second"));

        el.remove();
    }
}
