//! WebSocket-to-SSH terminal bridge.
//!
//! Provides bidirectional streaming between a WebSocket client and an SSH
//! interactive shell session. Supports terminal resize events and raw data
//! forwarding.

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::http::StatusCode;
use futures_util::{SinkExt, StreamExt};
use ops_pilot_core::audit::AuditTrail;
use ops_pilot_core::auth::AuthService;
use ops_pilot_core::ssh::{SshConnectionPool, SshError};
use russh::ChannelMsg;
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// JSON message sent by the client to request a terminal resize.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ResizeMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub cols: u32,
    pub rows: u32,
}

/// Parsed incoming WebSocket message from the client.
#[derive(Debug, PartialEq)]
pub enum TerminalMessage {
    /// Raw bytes to forward to the SSH channel's stdin.
    Data(Vec<u8>),
    /// Request to change the terminal dimensions.
    Resize { cols: u32, rows: u32 },
}

/// Attempt to parse a WebSocket text message into a `TerminalMessage`.
///
/// If the message is valid JSON matching the resize schema, returns
/// `TerminalMessage::Resize`. Otherwise treats it as data (UTF-8 encoded).
pub fn parse_message(text: &str) -> TerminalMessage {
    if let Ok(msg) = serde_json::from_str::<ResizeMessage>(text) {
        if msg.msg_type == "resize" {
            return TerminalMessage::Resize {
                cols: msg.cols,
                rows: msg.rows,
            };
        }
    }
    TerminalMessage::Data(text.as_bytes().to_vec())
}

/// Manages a single WebSocket ↔ SSH terminal session.
pub struct WebSocketHandler;

impl WebSocketHandler {
    /// Run the bidirectional bridge between `ws` and the SSH shell for `host_id`.
    ///
    /// Spawns two concurrent tasks:
    /// - **ws→ssh**: reads WebSocket messages, parses them, and forwards data /
    ///   resize requests into the SSH channel.
    /// - **ssh→ws**: reads SSH channel output and sends it back over the WebSocket.
    ///
    /// Returns when either side closes or on error. The SSH channel is cleaned up
    /// on exit.
    pub async fn run(
        ws: WebSocket,
        pool: Arc<SshConnectionPool>,
        host_id: String,
    ) -> Result<(), TerminalError> {
        let conn = pool.get(&host_id).await.map_err(TerminalError::Ssh)?;

        // Open an interactive shell channel.
        let handle = conn.handle.read().await;
        let channel = handle
            .channel_open_session()
            .await
            .map_err(|e| TerminalError::Channel(format!("failed to open channel: {}", e)))?;

        channel
            .request_pty(
                false,
                "xterm-256color",
                80,
                24,
                0,
                0,
                &[], // no special modes
            )
            .await
            .map_err(|e| TerminalError::Channel(format!("PTY request failed: {}", e)))?;

        channel
            .request_shell(true)
            .await
            .map_err(|e| TerminalError::Channel(format!("shell request failed: {}", e)))?;

        info!(host_id = %host_id, "SSH terminal session opened");

        let (mut ws_sender, mut ws_receiver) = ws.split();
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(64);

        // ssh→ws: forward channel output to the WebSocket.
        let ssh_to_ws = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if ws_sender.send(Message::Binary(msg.into())).await.is_err() {
                    break;
                }
            }
            let _ = ws_sender.close().await;
        });

        // ws→ssh: read WebSocket messages and forward to SSH.
        let mut channel = channel;
        while let Some(result) = ws_receiver.next().await {
            let ws_msg = match result {
                Ok(m) => m,
                Err(e) => {
                    warn!(error = %e, "WebSocket receive error");
                    break;
                }
            };

            match ws_msg {
                Message::Text(text) => match parse_message(&text) {
                    TerminalMessage::Data(data) => {
                        if channel.data(&data[..]).await.is_err() {
                            warn!("failed to write data to SSH channel");
                            break;
                        }
                    }
                    TerminalMessage::Resize { cols, rows } => {
                        debug!(cols, rows, "terminal resize");
                        if channel.window_change(cols, rows, 0, 0).await.is_err() {
                            warn!("failed to resize SSH terminal");
                        }
                    }
                },
                Message::Binary(data) => {
                    if channel.data(&data[..]).await.is_err() {
                        warn!("failed to write binary data to SSH channel");
                        break;
                    }
                }
                Message::Close(_) => {
                    info!("WebSocket closed by client");
                    break;
                }
                _ => continue,
            }
        }

        // Drain remaining SSH output.
        channel.eof().await.ok();
        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } => {
                    let _ = tx.send(data.to_vec()).await;
                }
                ChannelMsg::ExtendedData { data, .. } => {
                    let _ = tx.send(data.to_vec()).await;
                }
                ChannelMsg::Eof | ChannelMsg::ExitStatus { .. } => break,
                _ => continue,
            }
        }

        ssh_to_ws.abort();
        info!(host_id = %host_id, "SSH terminal session closed");
        Ok(())
    }
}

/// Errors produced by the WebSocket terminal handler.
#[derive(Debug, thiserror::Error)]
pub enum TerminalError {
    #[error("SSH error: {0}")]
    Ssh(#[from] SshError),

    #[error("channel error: {0}")]
    Channel(String),
}

/// Query parameters for the WebSocket terminal connection.
#[derive(Debug, Deserialize)]
pub struct TerminalQuery {
    pub token: Option<String>,
}

/// Axum handler that upgrades an HTTP request to a WebSocket and bridges it
/// to an SSH terminal session. Validates the JWT token from the `?token=`
/// query parameter before upgrading. Logs audit events for connect/disconnect.
pub async fn handle_ws_connection(
    ws: axum::extract::ws::WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<TerminalState>,
    axum::extract::Path(host_id): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<TerminalQuery>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let token = query.token.ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = state
        .auth
        .verify_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_id = claims.sub.clone();
    let audit = state.audit.clone();
    let resource = format!("host:{}", host_id);

    // Log successful SSH connect
    let _ = audit
        .log(&user_id, "ssh.connect", &resource, "success")
        .await;

    Ok(ws.on_upgrade(move |socket| async move {
        match WebSocketHandler::run(socket, state.pool, host_id.clone()).await {
            Ok(()) => {
                let _ = audit
                    .log(&user_id, "ssh.disconnect", &resource, "success")
                    .await;
            }
            Err(e) => {
                error!(host_id = %host_id, error = %e, "terminal session error");
                let _ = audit
                    .log(
                        &user_id,
                        "ssh.disconnect",
                        &resource,
                        &format!("failure: {e}"),
                    )
                    .await;
            }
        }
    }))
}

/// Shared state for the terminal WebSocket route.
#[derive(Clone)]
pub struct TerminalState {
    pub pool: Arc<SshConnectionPool>,
    pub auth: Arc<AuthService>,
    pub audit: Arc<AuditTrail>,
}

/// Build the terminal routes sub-router.
pub fn terminal_routes(
    pool: Arc<SshConnectionPool>,
    auth: Arc<AuthService>,
    audit: Arc<AuditTrail>,
) -> axum::Router {
    let state = TerminalState { pool, auth, audit };
    axum::Router::new()
        .route(
            "/api/terminal/{host_id}",
            axum::routing::get(handle_ws_connection),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resize_message() {
        let input = r#"{"type":"resize","cols":120,"rows":40}"#;
        let msg = parse_message(input);
        assert_eq!(
            msg,
            TerminalMessage::Resize {
                cols: 120,
                rows: 40
            }
        );
    }

    #[test]
    fn test_parse_resize_message_minimal() {
        let input = r#"{"type":"resize","cols":80,"rows":24}"#;
        let msg = parse_message(input);
        assert_eq!(msg, TerminalMessage::Resize { cols: 80, rows: 24 });
    }

    #[test]
    fn test_parse_data_message_plain_text() {
        let input = "ls -la\r";
        let msg = parse_message(input);
        assert_eq!(msg, TerminalMessage::Data(b"ls -la\r".to_vec()));
    }

    #[test]
    fn test_parse_data_message_json_wrong_type() {
        let input = r#"{"type":"input","data":"hello"}"#;
        let msg = parse_message(input);
        assert_eq!(
            msg,
            TerminalMessage::Data(br#"{"type":"input","data":"hello"}"#.to_vec())
        );
    }

    #[test]
    fn test_parse_data_message_invalid_json() {
        let input = "not json at all";
        let msg = parse_message(input);
        assert_eq!(msg, TerminalMessage::Data(b"not json at all".to_vec()));
    }

    #[test]
    fn test_parse_data_message_empty() {
        let input = "";
        let msg = parse_message(input);
        assert_eq!(msg, TerminalMessage::Data(b"".to_vec()));
    }

    #[test]
    fn test_resize_message_deserialize() {
        let json = r#"{"type":"resize","cols":200,"rows":50}"#;
        let msg: ResizeMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.msg_type, "resize");
        assert_eq!(msg.cols, 200);
        assert_eq!(msg.rows, 50);
    }

    #[test]
    fn test_resize_message_deserialize_invalid_type() {
        let json = r#"{"type":"data","cols":80,"rows":24}"#;
        let msg: ResizeMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.msg_type, "data");
        // parse_message should fall through to Data
        let parsed = parse_message(json);
        assert_eq!(parsed, TerminalMessage::Data(json.as_bytes().to_vec()));
    }

    #[test]
    fn test_parse_binary_like_text() {
        // Binary data encoded as text should be treated as data
        let input = "\x00\x01\x02";
        let msg = parse_message(input);
        assert_eq!(msg, TerminalMessage::Data(vec![0, 1, 2]));
    }

    #[test]
    fn test_terminal_error_display() {
        let err = TerminalError::Channel("test failure".to_string());
        assert!(err.to_string().contains("test failure"));

        let err = TerminalError::Ssh(SshError::Timeout);
        assert!(err.to_string().contains("SSH error"));
    }

    #[tokio::test]
    async fn test_terminal_state_clone() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let db = ops_pilot_core::db::Database::open_in_memory()
            .await
            .unwrap();
        let bus = ops_pilot_sdk::context::EventBus::new(16);
        let state = TerminalState {
            pool: Arc::new(SshConnectionPool::new()),
            auth: Arc::new(AuthService::new(pool, "test-secret".into())),
            audit: Arc::new(AuditTrail::new(&db, bus)),
        };
        let cloned = state.clone();
        assert_eq!(cloned.pool.connection_count(), 0);
    }

    #[test]
    fn test_parse_resize_with_extra_fields() {
        // Extra fields should be ignored by serde
        let input = r#"{"type":"resize","cols":132,"rows":50,"extra":"ignored"}"#;
        let msg = parse_message(input);
        assert_eq!(
            msg,
            TerminalMessage::Resize {
                cols: 132,
                rows: 50
            }
        );
    }

    #[test]
    fn test_parse_resize_missing_cols() {
        // Missing required field → serde fails → treated as data
        let input = r#"{"type":"resize","rows":24}"#;
        let msg = parse_message(input);
        assert_eq!(msg, TerminalMessage::Data(input.as_bytes().to_vec()));
    }

    #[test]
    fn test_terminal_query_deserialize() {
        let query: TerminalQuery = serde_json::from_str(r#"{"token":"abc123"}"#).unwrap();
        assert_eq!(query.token.as_deref(), Some("abc123"));
    }

    #[test]
    fn test_terminal_query_missing_token() {
        let query: TerminalQuery = serde_json::from_str("{}").unwrap();
        assert!(query.token.is_none());
    }
}
