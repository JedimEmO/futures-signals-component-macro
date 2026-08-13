use futures_signals_component_macro::component;

#[component(render_fn = my_cmp)]
struct MyCmp {
    #[signal]
    #[required]
    title: String,

    build: i32,
}

fn main() {}
