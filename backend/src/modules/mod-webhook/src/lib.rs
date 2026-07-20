//! mod-webhook: Webhook notification module.
//!
//! Dispatches events (alerts, audit logs, status changes) to external systems
//! via configurable webhook endpoints (Slack, DingTalk, Feishu, custom).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

// ── Types ───────────────────────────────────────────────────────────────────

/// Configuration for a single webhook endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookClient {
    /// Display name for this webhook (e.g. "slack-alerts").
    pub name: String,
    /// Target URL to POST events to.
    pub url: String,
    /// Optional secret sent as `X-Webhook-Secret` header for verification.
    pub secret: Option<String>,
    /// Number of retries on failure (0 = no retry).
    pub retry_count: u32,
}

/// Payload sent to webhook endpoints.
#[derive(Debug, Serialize)]
struct WebhookPayload {
    /// Event type as a string label.
    event_type: String,
    /// Serialized event data.
    event: OpsEvent,
    /// ISO-8601 timestamp.
    timestamp: String,
}

// ── Module ──────────────────────────────────────────────────────────────────

/// Webhook notification module — manages webhook registrations and
/// forwards events to external systems.
pub struct WebhookModule {
    clients: Arc<RwLock<HashMap<String, WebhookClient>>>,
    http: reqwest::Client,
}

impl Default for WebhookModule {
    fn default() -> Self {
        Self::new()
    }
}

impl WebhookModule {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            http: reqwest::Client::new(),
        }
    }

    /// Dispatch an event to a single webhook endpoint with retry.
    async fn dispatch_one(&self, client: &WebhookClient, event: &OpsEvent) -> anyhow::Result<()> {
        let payload = WebhookPayload {
            event_type: format!("{:?}", std::mem::discriminant(event)),
            event: event.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let body = serde_json::to_vec(&payload)?;
        let mut last_err = None;

        for attempt in 0..=client.retry_count {
            let mut req = self
                .http
                .post(&client.url)
                .header("Content-Type", "application/json")
                .body(body.clone());

            if let Some(ref secret) = client.secret {
                req = req.header("X-Webhook-Secret", secret.as_str());
            }

            match req.send().await {
                Ok(resp) if resp.status().is_success() => {
                    info!(
                        webhook = %client.name,
                        url = %client.url,
                        attempt,
                        "Webhook delivered successfully"
                    );
                    return Ok(());
                }
                Ok(resp) => {
                    let status = resp.status();
                    let err_msg = format!("HTTP {status}");
                    warn!(
                        webhook = %client.name,
                        url = %client.url,
                        attempt,
                        status = %status,
                        "Webhook delivery failed"
                    );
                    last_err = Some(anyhow::anyhow!(err_msg));
                }
                Err(e) => {
                    warn!(
                        webhook = %client.name,
                        url = %client.url,
                        attempt,
                        error = %e,
                        "Webhook delivery error"
                    );
                    last_err = Some(e.into());
                }
            }

            if attempt < client.retry_count {
                let delay_ms = 100 * 2u64.pow(attempt);
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("webhook delivery failed")))
    }

    /// Dispatch an event to all registered webhooks concurrently.
    async fn dispatch_all(&self, event: &OpsEvent) {
        let clients: Vec<WebhookClient> = {
            let map = self.clients.read().await;
            map.values().cloned().collect()
        };

        if clients.is_empty() {
            return;
        }

        let futures: Vec<_> = clients
            .iter()
            .map(|c| self.dispatch_one(c, event))
            .collect();

        let results = futures_util::future::join_all(futures).await;
        for (client, result) in clients.iter().zip(results) {
            if let Err(e) = result {
                error!(
                    webhook = %client.name,
                    error = %e,
                    "Failed to deliver event after retries"
                );
            }
        }
    }
}

#[async_trait]
impl OpsModule for WebhookModule {
    fn name(&self) -> &str {
        "mod-webhook"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "Webhook notification — dispatch events to Slack, DingTalk, Feishu, etc."
    }

    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "webhook_register".into(),
                description: "Register a new webhook endpoint".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Display name for this webhook (e.g. 'slack-alerts')"
                        },
                        "url": {
                            "type": "string",
                            "description": "Target URL to POST events to"
                        },
                        "secret": {
                            "type": "string",
                            "description": "Optional secret for X-Webhook-Secret header"
                        },
                        "retry_count": {
                            "type": "integer",
                            "description": "Number of retries on failure (default: 3)",
                            "default": 3
                        }
                    },
                    "required": ["name", "url"]
                }),
            },
            ToolDefinition {
                name: "webhook_unregister".into(),
                description: "Remove a registered webhook endpoint".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the webhook to remove"
                        }
                    },
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "webhook_send".into(),
                description: "Send a test message to a specific webhook".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the webhook to send to"
                        },
                        "message": {
                            "type": "string",
                            "description": "Message content to send"
                        }
                    },
                    "required": ["name", "message"]
                }),
            },
            ToolDefinition {
                name: "webhook_list".into(),
                description: "List all registered webhook endpoints".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ]
    }

    async fn execute(
        &self,
        _ctx: &ModuleContext,
        tool: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        match tool {
            "webhook_register" => {
                let name = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'name' parameter"))?
                    .to_string();
                let url = params["url"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'url' parameter"))?
                    .to_string();
                let secret = params["secret"].as_str().map(String::from);
                let retry_count = params["retry_count"].as_u64().unwrap_or(3) as u32;

                let client = WebhookClient {
                    name: name.clone(),
                    url,
                    secret,
                    retry_count,
                };

                let mut clients = self.clients.write().await;
                clients.insert(name.clone(), client);

                info!(webhook = %name, "Webhook registered");
                Ok(serde_json::json!({
                    "status": "registered",
                    "name": name
                }))
            }
            "webhook_unregister" => {
                let name = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'name' parameter"))?;

                let mut clients = self.clients.write().await;
                let removed = clients.remove(name).is_some();

                if removed {
                    info!(webhook = %name, "Webhook unregistered");
                    Ok(serde_json::json!({
                        "status": "removed",
                        "name": name
                    }))
                } else {
                    Ok(serde_json::json!({
                        "status": "not_found",
                        "name": name
                    }))
                }
            }
            "webhook_send" => {
                let name = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'name' parameter"))?;
                let message = params["message"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'message' parameter"))?;

                let client = {
                    let clients = self.clients.read().await;
                    clients
                        .get(name)
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!("webhook '{}' not found", name))?
                };

                let event = OpsEvent::Custom {
                    source: "webhook_send".into(),
                    kind: "manual".into(),
                    payload: serde_json::json!({ "message": message }),
                };

                self.dispatch_one(&client, &event).await?;

                Ok(serde_json::json!({
                    "status": "sent",
                    "name": name
                }))
            }
            "webhook_list" => {
                let clients = self.clients.read().await;
                let list: Vec<&WebhookClient> = clients.values().collect();
                Ok(serde_json::to_value(&list)?)
            }
            _ => Err(anyhow::anyhow!("unknown tool: {}", tool)),
        }
    }

    async fn on_event(&self, _ctx: &ModuleContext, event: &OpsEvent) -> Option<ModuleAction> {
        // Auto-forward AlertTriggered events to all registered webhooks
        if matches!(event, OpsEvent::AlertTriggered { .. }) {
            info!("AlertTriggered event received, forwarding to webhooks");
            self.dispatch_all(event).await;
        }
        None
    }

    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
        let count = self.clients.read().await.len();
        if count == 0 {
            HealthStatus::Degraded {
                reason: "no webhooks registered".into(),
            }
        } else {
            HealthStatus::Healthy
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ops_pilot_sdk::context::{EventBus, ModuleContext};
    use std::path::PathBuf;

    async fn make_ctx(name: &str) -> ModuleContext {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        ModuleContext::new(
            Arc::new(pool),
            EventBus::new(16),
            PathBuf::from(format!("/tmp/{name}")),
            name.to_string(),
        )
    }

    #[tokio::test]
    async fn test_module_metadata() {
        let m = WebhookModule::new();
        assert_eq!(m.name(), "mod-webhook");
        assert_eq!(m.version(), "0.1.0");
        assert!(m.description().contains("Webhook"));
    }

    #[tokio::test]
    async fn test_tools_registered() {
        let m = WebhookModule::new();
        let tools = m.tools();
        assert_eq!(tools.len(), 4);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"webhook_register"));
        assert!(names.contains(&"webhook_unregister"));
        assert!(names.contains(&"webhook_send"));
        assert!(names.contains(&"webhook_list"));
    }

    #[tokio::test]
    async fn test_register_and_list() {
        let m = WebhookModule::new();
        let ctx = make_ctx("test").await;

        let result = m
            .execute(
                &ctx,
                "webhook_register",
                serde_json::json!({
                    "name": "test-hook",
                    "url": "https://example.com/hook",
                    "secret": "s3cret",
                    "retry_count": 2
                }),
            )
            .await
            .unwrap();
        assert_eq!(result["status"], "registered");

        let list = m
            .execute(&ctx, "webhook_list", serde_json::json!({}))
            .await
            .unwrap();
        let arr = list.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["name"], "test-hook");
    }

    #[tokio::test]
    async fn test_unregister() {
        let m = WebhookModule::new();
        let ctx = make_ctx("test").await;

        m.execute(
            &ctx,
            "webhook_register",
            serde_json::json!({ "name": "h1", "url": "https://example.com" }),
        )
        .await
        .unwrap();

        let result = m
            .execute(
                &ctx,
                "webhook_unregister",
                serde_json::json!({ "name": "h1" }),
            )
            .await
            .unwrap();
        assert_eq!(result["status"], "removed");

        let list = m
            .execute(&ctx, "webhook_list", serde_json::json!({}))
            .await
            .unwrap();
        assert!(list.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_health_check_no_webhooks() {
        let m = WebhookModule::new();
        let ctx = make_ctx("test").await;
        let status = m.health_check(&ctx).await;
        assert!(matches!(status, HealthStatus::Degraded { .. }));
    }

    #[tokio::test]
    async fn test_health_check_with_webhook() {
        let m = WebhookModule::new();
        let ctx = make_ctx("test").await;

        m.execute(
            &ctx,
            "webhook_register",
            serde_json::json!({ "name": "h1", "url": "https://example.com" }),
        )
        .await
        .unwrap();

        let status = m.health_check(&ctx).await;
        assert!(matches!(status, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let m = WebhookModule::new();
        let ctx = make_ctx("test").await;
        let result = m.execute(&ctx, "nonexistent", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_on_event_non_alert() {
        let m = WebhookModule::new();
        let ctx = make_ctx("test").await;
        let event = OpsEvent::AuditLog {
            user: "u".into(),
            action: "a".into(),
            resource: "r".into(),
            outcome: "ok".into(),
        };
        // Should not panic, returns None
        let action = m.on_event(&ctx, &event).await;
        assert!(action.is_none());
    }

    #[tokio::test]
    async fn test_on_event_alert_no_webhooks() {
        let m = WebhookModule::new();
        let ctx = make_ctx("test").await;
        let event = OpsEvent::AlertTriggered {
            severity: "critical".into(),
            message: "test alert".into(),
        };
        // Should not panic even with no webhooks
        let action = m.on_event(&ctx, &event).await;
        assert!(action.is_none());
    }
}
