//! DingTalk (钉钉) notification sender.
//!
//! Supports text, markdown, and action card message types via webhook URL.
//! HMAC-SHA256 signing when a secret is provided.

use serde::Serialize;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize)]
pub struct ActionBtn {
    pub title: String,
    pub action_url: String,
}

/// DingTalk notification sender.
pub struct DingTalkNotifier {
    http: reqwest::Client,
}

impl DingTalkNotifier {
    pub fn new() -> Self {
        Self { http: reqwest::Client::new() }
    }

    /// Sign the webhook URL with HMAC-SHA256 if a secret is provided.
    fn sign_url(webhook_url: &str, secret: Option<&str>) -> String {
        let Some(secret) = secret else {
            return webhook_url.to_string();
        };
        let timestamp = chrono::Utc::now().timestamp_millis();
        let string_to_sign = format!("{}\n{}", timestamp, secret);
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
        mac.update(string_to_sign.as_bytes());
        let sign = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &mac.finalize().into_bytes());
        format!("{}&timestamp={}&sign={}", webhook_url, timestamp, sign)
    }

    /// Send a plain text message.
    pub async fn send_text(&self, webhook_url: &str, text: &str, secret: Option<&str>) -> anyhow::Result<()> {
        let url = Self::sign_url(webhook_url, secret);
        let payload = serde_json::json!({
            "msgtype": "text",
            "text": { "content": text }
        });
        self.post(&url, &payload).await
    }

    /// Send a markdown message.
    pub async fn send_markdown(&self, webhook_url: &str, title: &str, text: &str, secret: Option<&str>) -> anyhow::Result<()> {
        let url = Self::sign_url(webhook_url, secret);
        let payload = serde_json::json!({
            "msgtype": "markdown",
            "markdown": { "title": title, "text": text }
        });
        self.post(&url, &payload).await
    }

    /// Send an action card message.
    pub async fn send_action_card(&self, webhook_url: &str, title: &str, text: &str, btns: Vec<ActionBtn>, secret: Option<&str>) -> anyhow::Result<()> {
        let url = Self::sign_url(webhook_url, secret);
        let single_title = btns.first().map(|b| b.title.clone()).unwrap_or_default();
        let single_url = btns.first().map(|b| b.action_url.clone()).unwrap_or_default();
        let payload = serde_json::json!({
            "msgtype": "actionCard",
            "actionCard": {
                "title": title,
                "text": text,
                "btnOrientation": "0",
                "singleTitle": single_title,
                "singleURL": single_url,
            }
        });
        self.post(&url, &payload).await
    }

    async fn post(&self, url: &str, payload: &serde_json::Value) -> anyhow::Result<()> {
        let resp = self.http.post(url)
            .json(payload)
            .send()
            .await?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await.unwrap_or_default();

        if status.is_success() && body.get("errcode").and_then(|v| v.as_i64()) == Some(0) {
            Ok(())
        } else {
            let errcode = body.get("errcode").and_then(|v| v.as_i64()).unwrap_or(-1);
            let errmsg = body.get("errmsg").and_then(|v| v.as_str()).unwrap_or("unknown");
            Err(anyhow::anyhow!("DingTalk error {}: {}", errcode, errmsg))
        }
    }
}

impl Default for DingTalkNotifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_url_without_secret() {
        let url = DingTalkNotifier::sign_url("https://oapi.dingtalk.com/robot/send?access_token=abc", None);
        assert_eq!(url, "https://oapi.dingtalk.com/robot/send?access_token=abc");
    }

    #[test]
    fn test_sign_url_with_secret() {
        let url = DingTalkNotifier::sign_url("https://oapi.dingtalk.com/robot/send?access_token=abc", Some("SECsecret"));
        assert!(url.contains("timestamp="));
        assert!(url.contains("sign="));
        assert!(url.starts_with("https://oapi.dingtalk.com/robot/send?access_token=abc&timestamp="));
    }
}
