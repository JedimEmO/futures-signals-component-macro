use futures_signals_component_macro::component;

#[component(render_fn = my_cmp)]
struct MyCmp {
    #[signal_vec]
    #[into]
    items: i32,
}

fn main() {}
