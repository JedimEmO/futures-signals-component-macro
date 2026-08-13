use futures_signals_component_macro::component;

#[component(render_fn = my_cmp)]
struct MyCmp {
    #[required]
    #[default(42)]
    x: i32,
}

fn main() {}
