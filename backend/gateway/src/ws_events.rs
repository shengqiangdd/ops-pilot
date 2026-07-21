use axum::extract::ws::{Message, WebSocket};
use axum::response::IntoResponse;
use futures::stream::StreamExt;
use futures_util::SinkExt;
use ops_pilot_sdk::global_event_bus;
use serde_json::json;
use tokio_stream::wrappers::BroadcastStream;

/// WebSocket 事件广播处理器
pub async fn ws_events_handler(ws: axum::extract::ws::WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    let (mut sender, _receiver) = socket.split();

    // 订阅全局 EventBus
    let rx = global_event_bus().subscribe();
    let mut stream = BroadcastStream::new(rx);

    // 发送心跳保持连接
    let ping_interval = tokio::time::interval(std::time::Duration::from_secs(30));
    tokio::pin!(ping_interval);

    loop {
        tokio::select! {
            event = stream.next() => {
                match event {
                    Some(Ok(ops_event)) => {
                        let msg = serde_json::to_string(&json!({
                            "type": "event",
                            "data": ops_event,
                        }))
                        .unwrap_or_default();
                        if sender.send(Message::Text(msg.into())).await.is_err() {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            _ = ping_interval.tick() => {
                if sender.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
            }
        }
    }
}
