//! mod-webhook: Webhook notification module.
//!
//! Dispatches events to external systems via:
//! - Generic webhooks (Slack, custom)
//! - DingTalk (钉钉) robot
//! - WeCom (企业微信) robot
//! - Email (SMTP)

pub mod dingtalk;
pub mod email;
pub mod wecom;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookClient {
    pub name: String,
    pub url: String,
    pub secret: Option<String>,
    pub retry_count: u32,
}

#[derive(Debug, Serialize)]
struct WebhookPayload {
    event_type: String,
    event: OpsEvent,
    timestamp: String,
}

// ── Module ──────────────────────────────────────────────────────────────────

pub struct WebhookModule {
    clients: Arc<RwLock<HashMap<String, WebhookClient>>>,
    http: reqwest::Client,
    dingtalk: dingtalk::DingTalkNotifier,
    wecom: wecom::WeComNotifier,
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
            dingtalk: dingtalk::DingTalkNotifier::new(),
            wecom: wecom::WeComNotifier::new(),
        }
    }

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
                    info!(webhook = %client.name, attempt, "Webhook delivered");
                    return Ok(());
                }
                Ok(resp) => {
                    let status = resp.status();
                    warn!(webhook = %client.name, attempt, status = %status, "Webhook delivery failed");
                    last_err = Some(anyhow::anyhow!("HTTP {status}"));
                }
                Err(e) => {
                    warn!(webhook = %client.name, attempt, error = %e, "Webhook delivery error");
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

    async fn dispatch_all(&self, event: &OpsEvent) {
        let clients: Vec<WebhookClient> = {
            let map = self.clients.read().await;
            map.values().cloned().collect()
        };
        if clients.is_empty() { return; }

        let futures: Vec<_> = clients.iter().map(|c| self.dispatch_one(c, event)).collect();
        let results = futures_util::future::join_all(futures).await;
        for (client, result) in clients.iter().zip(results) {
            if let Err(e) = result {
                error!(webhook = %client.name, error = %e, "Failed after retries");
            }
        }
    }
}

#[async_trait]
impl OpsModule for WebhookModule {
    fn name(&self) -> &str { "mod-webhook" }
    fn version(&self) -> &str { "0.1.0" }
    fn description(&self) -> &str { "Notifications — DingTalk, WeCom, Email, webhooks" }
    fn dependencies(&self) -> Vec<&str> { vec![] }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "webhook_register".into(),
                description: "Register a webhook endpoint".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "url": {"type": "string"},
                        "secret": {"type": "string"},
                        "retry_count": {"type": "integer", "default": 3}
                    },
                    "required": ["name", "url"]
                }),
            },
            ToolDefinition {
                name: "webhook_unregister".into(),
                description: "Remove a webhook endpoint".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {"name": {"type": "string"}},
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "webhook_send".into(),
                description: "Send a test message to a webhook".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "message": {"type": "string"}
                    },
                    "required": ["name", "message"]
                }),
            },
            ToolDefinition {
                name: "webhook_list".into(),
                description: "List all webhooks".into(),
                parameters: serde_json::json!({"type": "object", "properties": {}}),
            },
            ToolDefinition {
                name: "dingtalk_send".into(),
                description: "Send a message via DingTalk (钉钉) robot".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "webhook_url": {"type": "string", "description": "DingTalk robot webhook URL"},
                        "secret": {"type": "string", "description": "Optional HMAC secret for signing"},
                        "msg_type": {"type": "string", "enum": ["text", "markdown"], "default": "text"},
                        "title": {"type": "string", "description": "Title (for markdown)"},
                        "content": {"type": "string", "description": "Message content"}
                    },
                    "required": ["webhook_url", "content"]
                }),
            },
            ToolDefinition {
                name: "wecom_send".into(),
                description: "Send a message via WeCom (企业微信) robot".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "webhook_url": {"type": "string", "description": "WeCom robot webhook URL"},
                        "msg_type": {"type": "string", "enum": ["text", "markdown"], "default": "text"},
                        "content": {"type": "string", "description": "Message content"}
                    },
                    "required": ["webhook_url", "content"]
                }),
            },
            ToolDefinition {
                name: "email_send".into(),
                description: "Send an email via SMTP".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "to": {"type": "array", "items": {"type": "string"}, "description": "Recipient email addresses"},
                        "subject": {"type": "string"},
                        "body": {"type": "string"}
                    },
                    "required": ["to", "subject", "body"]
                }),
            },
        ]
    }

    async fn execute(&self, _ctx: &ModuleContext, tool: &str, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        match tool {
            "webhook_register" => {
                let name = params["name"].as_str().ok_or_else(|| anyhow::anyhow!("missing 'name'"))?.to_string();
                let url = params["url"].as_str().ok_or_else(|| anyhow::anyhow!("missing 'url'"))?.to_string();
                let secret = params["secret"].as_str().map(String::from);
                let retry_count = params["retry_count"].as_u64().unwrap_or(3) as u32;
                let client = WebhookClient { name: name.clone(), url, secret, retry_count };
                self.clients.write().await.insert(name.clone(), client);
                info!(webhook = %name, "Registered");
                Ok(serde_json::json!({"status": "registered", "name": name}))
            }
            "webhook_unregister" => {
                let name = params["name"].as_str().ok_or_else(|| anyhow::anyhow!("missing 'name'"))?;
                let removed = self.clients.write().await.remove(name).is_some();
                Ok(serde_json::json!({"status": if removed { "removed" } else { "not_found" }, "name": name}))
            }
            "webhook_send" => {
                let name = params["name"].as_str().ok_or_else(|| anyhow::anyhow!("missing 'name'"))?;
                let message = params["message"].as_str().ok_or_else(|| anyhow::anyhow!("missing 'message'"))?;
                let client = { self.clients.read().await.get(name).cloned()
                    .ok_or_else(|| anyhow::anyhow!("webhook '{}' not found", name))? };
                let event = OpsEvent::Custom { source: "webhook_send".into(), kind: "manual".into(), payload: serde_json::json!({"message": message}) };
                self.dispatch_one(&client, &event).await?;
                Ok(serde_json::json!({"status": "sent", "name": name}))
            }
            "webhook_list" => {
                let clients = self.clients.read().await;
                let list: Vec<WebhookClient> = clients.values().cloned().collect();
                Ok(serde_json::to_value(&list)?)
            }
            "dingtalk_send" => {
                let url = params["webhook_url"].as_str().ok_or_else(|| anyhow::anyhow!("missing 'webhook_url'"))?;
                let content = params["content"].as_str().ok_or_else(|| anyhow::anyhow!("missing 'content'"))?;
                let secret = params["secret"].as_str();
                let msg_type = params["msg_type"].as_str().unwrap_or("text");
                let title = params["title"].as_str().unwrap_or("OpsPilot Alert");

                match msg_type {
                    "markdown" => self.dingtalk.send_markdown(url, title, content, secret).await?,
                    _ => self.dingtalk.send_text(url, content, secret).await?,
                }
                Ok(serde_json::json!({"status": "sent", "channel": "dingtalk"}))
            }
            "wecom_send" => {
                let url = params["webhook_url"].as_str().ok_or_else(|| anyhow::anyhow!("missing 'webhook_url'"))?;
                let content = params["content"].as_str().ok_or_else(|| anyhow::anyhow!("missing 'content'"))?;
                let msg_type = params["msg_type"].as_str().unwrap_or("text");

                match msg_type {
                    "markdown" => self.wecom.send_markdown(url, content).await?,
                    _ => self.wecom.send_text(url, content).await?,
                }
                Ok(serde_json::json!({"status": "sent", "channel": "wecom"}))
            }
            "email_send" => {
                let to: Vec<String> = params["to"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .ok_or_else(|| anyhow::anyhow!("missing 'to'"))?;
                let subject = params["subject"].as_str().ok_or_else(|| anyhow::anyhow!("missing 'subject'"))?;
                let body = params["body"].as_str().ok_or_else(|| anyhow::anyhow!("missing 'body'"))?;

                let notifier = email::EmailNotifier::from_env()
                    .ok_or_else(|| anyhow::anyhow!("SMTP not configured (set SMTP_HOST env var)"))?;
                notifier.send(&to, subject, body).await?;
                Ok(serde_json::json!({"status": "sent", "channel": "email"}))
            }
            _ => Err(anyhow::anyhow!("unknown tool: {}", tool)),
        }
    }

    async fn on_event(&self, _ctx: &ModuleContext, event: &OpsEvent) -> Option<ModuleAction> {
        if matches!(event, OpsEvent::AlertTriggered { .. }) {
            info!("AlertTriggered → forwarding to webhooks");
            self.dispatch_all(event).await;
        }
        None
    }

    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
        let count = self.clients.read().await.len();
        if count == 0 { HealthStatus::Degraded { reason: "no webhooks registered".into() } }
        else { HealthStatus::Healthy }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ops_pilot_sdk::context::{EventBus, ModuleContext};
    use std::path::PathBuf;

    async fn make_ctx(name: &str) -> ModuleContext {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        ModuleContext::new(Arc::new(pool), EventBus::new(16), PathBuf::from(format!("/tmp/{name}")), name.to_string())
    }

    #[tokio::test]
    async fn test_module_metadata() {
        let m = WebhookModule::new();
        assert_eq!(m.name(), "mod-webhook");
        assert!(m.description().contains("DingTalk"));
    }

    #[tokio::test]
    async fn test_tools_registered() {
        let m = WebhookModule::new();
        let tools = m.tools();
        assert_eq!(tools.len(), 7);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"dingtalk_send"));
        assert!(names.contains(&"wecom_send"));
        assert!(names.contains(&"email_send"));
        assert!(names.contains(&"webhook_register"));
    }

    #[tokio::test]
    async fn test_register_and_list() {
        let m = WebhookModule::new();
        let ctx = make_ctx("test").await;
        let result = m.execute(&ctx, "webhook_register", serde_json::json!({"name": "h1", "url": "https://example.com"})).await.unwrap();
        assert_eq!(result["status"], "registered");
        let list = m.execute(&ctx, "webhook_list", serde_json::json!({})).await.unwrap();
        assert_eq!(list.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let m = WebhookModule::new();
        let ctx = make_ctx("test").await;
        assert!(m.execute(&ctx, "nonexistent", serde_json::json!({})).await.is_err());
    }

    #[tokio::test]
    async fn test_health_check() {
        let m = WebhookModule::new();
        let ctx = make_ctx("test").await;
        assert!(matches!(m.health_check(&ctx).await, HealthStatus::Degraded { .. }));
        m.execute(&ctx, "webhook_register", serde_json::json!({"name": "h1", "url": "https://x.com"})).await.unwrap();
        assert!(matches!(m.health_check(&ctx).await, HealthStatus::Healthy));
    }
}
