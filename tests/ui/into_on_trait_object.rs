use futures_signals_component_macro::component;

#[component(render_fn = my_cmp)]
struct MyCmp {
    #[into]
    handler: dyn Fn() + 'static,
}

fn main() {}
