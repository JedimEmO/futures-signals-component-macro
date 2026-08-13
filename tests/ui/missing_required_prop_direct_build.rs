use futures_signals_component_macro::component;

#[component(render_fn = my_cmp)]
struct MyCmp {
    #[signal]
    #[required]
    title: String,

    #[signal]
    #[default(0)]
    count: i32,
}

fn my_cmp(props: MyCmpProps) {
    let _ = props.title;
}

fn main() {
    // Setting only the optional prop leaves the builder in the missing-title state.
    let _props = MyCmpProps::new().count(4).build();
}
