use futures_signals_component_macro::component;

#[component(render_fn = my_cmp)]
struct MyCmp {
    #[signal]
    #[required]
    title: String,
}

fn my_cmp(props: MyCmpProps) {
    let _ = props.title;
}

fn main() {
    my_cmp!({});
}
