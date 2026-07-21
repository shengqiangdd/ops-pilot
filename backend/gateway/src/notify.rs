//! Notification delivery backend — Email (SMTP) + Webhook + Slack.
//!
//! Reads channel config from the `notification_channels` DB table and
//! dispatches messages accordingly.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;

/// A persisted notification channel (matching the frontend table schema).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotifyChannel {
    pub id: String,
    pub name: String,
    pub channel_type: String, // "email", "webhook", "slack"
    pub config: String,       // JSON config — maps to `config` column in `notification_channels`
    pub enabled: bool,
    pub created_at: String,
}

/// Deliver a message to all enabled channels.
pub async fn dispatch_notification(
    pool: &SqlitePool,
    title: &str,
    message: &str,
    severity: &str, // "info", "warning", "critical"
) -> Result<Vec<String>> {
    let channels = sqlx::query_as::<_, NotifyChannel>(
        "SELECT id, name, channel_type, config, enabled, created_at FROM notification_channels WHERE enabled = 1",
    )
    .fetch_all(pool)
    .await?;

    let mut results = Vec::new();
    for ch in &channels {
        match ch.channel_type.as_str() {
            "email" => {
                let r = send_email(ch, title, message).await;
                results.push(format!("email[{}]: {:?}", ch.name, r));
            }
            "webhook" => {
                let r = send_webhook(ch, title, message, severity).await;
                results.push(format!("webhook[{}]: {:?}", ch.name, r));
            }
            "slack" => {
                let r = send_slack(ch, title, message).await;
                results.push(format!("slack[{}]: {:?}", ch.name, r));
            }
            _ => {}
        }
    }
    Ok(results)
}

async fn send_email(ch: &NotifyChannel, title: &str, message: &str) -> Result<()> {
    let config: HashMap<String, String> = serde_json::from_str(&ch.config)?;
    let smtp_host = config.get("smtp_host").cloned().unwrap_or_default();
    let smtp_port = config
        .get("smtp_port")
        .and_then(|p| p.parse().ok())
        .unwrap_or(587);
    let smtp_user = config.get("smtp_user").cloned().unwrap_or_default();
    let smtp_pass = config.get("smtp_pass").cloned().unwrap_or_default();
    let _from = config.get("from").cloned().unwrap_or_default();
    let to = config.get("to").cloned().unwrap_or_default();

    if smtp_host.is_empty() || to.is_empty() {
        return Err(anyhow::anyhow!("Email config incomplete"));
    }

    use lettre::{
        message::Mailbox,
        transport::smtp::authentication::Credentials,
        AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    };

    let email = Message::builder()
        .from(
            _from
                .parse::<Mailbox>()
                .unwrap_or_else(|_| "OpsPilot <noreply@opspilot>".parse().unwrap()),
        )
        .to(to.parse()?)
        .subject(format!("[OpsPilot] {}", title))
        .body(message.to_string())?;

    let creds = Credentials::new(smtp_user, smtp_pass);
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)?
        .port(smtp_port)
        .credentials(creds)
        .build();

    mailer.send(email).await?;
    Ok(())
}

async fn send_webhook(
    ch: &NotifyChannel,
    title: &str,
    message: &str,
    severity: &str,
) -> Result<()> {
    let config: HashMap<String, String> = serde_json::from_str(&ch.config)?;
    let url = config.get("url").cloned().unwrap_or_default();
    if url.is_empty() {
        return Err(anyhow::anyhow!("Webhook URL not configured"));
    }

    let body = serde_json::json!({
        "title": title,
        "message": message,
        "severity": severity,
        "source": "ops-pilot",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("Webhook returned {}", resp.status()));
    }
    Ok(())
}

async fn send_slack(ch: &NotifyChannel, title: &str, message: &str) -> Result<()> {
    let config: HashMap<String, String> = serde_json::from_str(&ch.config)?;
    let webhook_url = config.get("webhook_url").cloned().unwrap_or_default();
    if webhook_url.is_empty() {
        return Err(anyhow::anyhow!("Slack webhook URL not configured"));
    }

    let body = serde_json::json!({
        "text": format!("*[OpsPilot]* {}\n{}", title, message),
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&webhook_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("Slack returned {}", resp.status()));
    }
    Ok(())
}
