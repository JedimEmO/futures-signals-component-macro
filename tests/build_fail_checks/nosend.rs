use futures_signals::signal::Signal;
use futures_signals_component_macro::component;

#[component(render_fn = render_send)]
struct NeedsSend<T: Send = (), TNotSend: Clone = ()> {
    #[signal]
    send_me: T,

    #[signal]
    don_not_send_me: TNotSend,
}

fn render_send(props: impl NeedsSendPropsTrait + 'static) -> i32 {
    let NeedsSendProps {
        send_me,
        don_not_send_me
    } = props.take();

    consume_send(don_not_send_me.unwrap());

    42
}

fn consume_send(_: impl Signal<Item=impl Send>) {}

fn main() {}