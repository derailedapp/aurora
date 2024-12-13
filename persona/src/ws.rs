use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::Response,
};
use pulsar::Consumer;

use futures_util::TryStreamExt;

pub async fn handle_relay(
    ws: WebSocketUpgrade,
    State(state): State<crate::state::State>,
) -> Response {
    ws.on_upgrade(|socket| handle_relay_impl(socket, state))
}

async fn handle_relay_impl(mut ws: WebSocket, state: crate::state::State) {
    let consumer: Result<Consumer<String, _>, pulsar::Error> = state
        .plsr
        .consumer()
        .with_topic("test")
        .with_consumer_name("test_consumer")
        .with_subscription_type(pulsar::SubType::Exclusive)
        .with_subscription("test_subscription")
        .build()
        .await;

    if let Ok(mut consumer) = consumer {
        while let Ok(Some(msg)) = consumer.try_next().await {
            _ = ws
                .send(axum::extract::ws::Message::Text(msg.deserialize().unwrap()))
                .await;
        }
    } else {
        let _ = ws.close().await;
    }
}
