use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_framepace::FramepacePlugin;
use haalka::prelude::*;
use futures_signals_component_macro::component;

#[component(render_fn = cool_button)]
struct CoolButton<FOnClick: (FnMut() -> ()) + Send + Sync = fn() -> ()> {
    /// A label field which can be used as either a signal or a constant value
    /// This will cause the cool_button! macro to have both .label() and .label_signal() methods
    /// for providing a label value
    #[signal]
    #[send] // We need to explicitly declare the label signal as Send, since we don't know that String is send compile time when generating the code
    #[default("".to_string())]
    label: String,

    #[default(|| {})]
    on_click: FOnClick,
}

fn cool_button(props: impl CoolButtonPropsTrait + 'static) -> impl Element {
    // Extract the prop fields into variables. This trick allows us to use sized types without
    // a generic type explosion
    let CoolButtonProps {
        label,
        on_click
    } = props.take();

    let hovered = Mutable::new(false);

    let text_details = map_ref! {
        let hovered = hovered.signal(),
        let label = label => {
            let color = if *hovered {
                tailwind::EMERALD_500
            } else {
                tailwind::EMERALD_50
            };

            (Color::Srgba(color), label.clone())
        }
    };

    Stack::<NodeBundle>::new()
        .background_color(BackgroundColor(Color::Srgba(tailwind::SLATE_800)))
        .style(Style {
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        })
        .on_click(on_click)
        .hovered_sync(hovered.clone())
        .align_content(Align::center())
        .layer(El::<TextBundle>::new()
            .text_signal(text_details.map(move |(color, label)| {
                Text::from_section(
                    label,
                    TextStyle {
                        font_size: 20.0,
                        color,
                        ..default()
                    },
                )
            })))
}

/// This system initialized the UI root in the world
fn ui_root(world: &mut World) {
    let label = Mutable::new("Click me!".to_string());

    // Create a button instance. The return value of cool_button! is the return value of its render_fn (cool_button() -> impl Element in this case)
    let button = cool_button!({
        .on_click(clone!((label) move || {
            label.set("Yay, I am clicked!".to_string())
        }))
        .label_signal(label.signal_cloned())
    });

    let inert_button = cool_button!({
        .label("I do nothing".to_string())
    });

    El::<NodeBundle>::new()
        .width(Val::Percent(100.0))
        .height(Val::Percent(100.0))
        .align_content(Align::center())
        .child(Column::<NodeBundle>::new()
            .item(button)
            .item(inert_button)
        )
        .spawn(world);
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            HaalkaPlugin,
        ))

        .add_plugins(FramepacePlugin)
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.2)))
        .add_systems(Startup, (setup, ui_root).chain())
        .run();
}