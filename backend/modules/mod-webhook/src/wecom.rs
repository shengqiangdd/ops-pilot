//! 企业微信 (WeCom) notification sender.
//!
//! Sends messages via WeCom group bot webhook URL.
//! Supports text, markdown, and news (article) message types.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Article {
    pub title: String,
    pub description: String,
    pub url: String,
    pub picurl: Option<String>,
}

/// WeCom group bot notification sender.
pub struct WeComNotifier {
    http: reqwest::Client,
}

impl WeComNotifier {
    pub fn new() -> Self {
        Self { http: reqwest::Client::new() }
    }

    /// Send a plain text message.
    pub async fn send_text(&self, webhook_url: &str, content: &str) -> anyhow::Result<()> {
        let payload = serde_json::json!({
            "msgtype": "text",
            "text": { "content": content }
        });
        self.post(webhook_url, &payload).await
    }

    /// Send a markdown message.
    pub async fn send_markdown(&self, webhook_url: &str, content: &str) -> anyhow::Result<()> {
        let payload = serde_json::json!({
            "msgtype": "markdown",
            "markdown": { "content": content }
        });
        self.post(webhook_url, &payload).await
    }

    /// Send a news (article card) message.
    pub async fn send_news(&self, webhook_url: &str, articles: Vec<Article>) -> anyhow::Result<()> {
        let items: Vec<serde_json::Value> = articles.iter().map(|a| {
            let mut item = serde_json::json!({
                "title": a.title,
                "description": a.description,
                "url": a.url,
            });
            if let Some(ref picurl) = a.picurl {
                item["picurl"] = serde_json::json!(picurl);
            }
            item
        }).collect();

        let payload = serde_json::json!({
            "msgtype": "news",
            "news": { "articles": items }
        });
        self.post(webhook_url, &payload).await
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
            Err(anyhow::anyhow!("WeCom error {}: {}", errcode, errmsg))
        }
    }
}

impl Default for WeComNotifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_text_invalid_url() {
        let notifier = WeComNotifier::new();
        let result = notifier.send_text("http://localhost:1", "test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_markdown_invalid_url() {
        let notifier = WeComNotifier::new();
        let result = notifier.send_markdown("http://localhost:1", "**test**").await;
        assert!(result.is_err());
    }
}
