use futures_signals_component_macro::component;

#[component(render_fn = my_cmp)]
struct MyCmp {
    #[signal]
    #[signal_vec]
    x: i32,
}

fn main() {}
