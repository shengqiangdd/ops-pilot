//! Webhook delivery retry queue with dead-letter support.
//!
//! Failed deliveries are retried up to `MAX_RETRIES` times with
//! exponential backoff.  When all retries are exhausted the message
//! moves to a dead-letter table for manual inspection.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::time::Duration;
use tokio::time::sleep;
use tracing;

const MAX_RETRIES: u32 = 5;
const BASE_DELAY_MS: u64 = 1_000;

/// A queued delivery.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QueuedDelivery {
    pub id: String,
    pub channel_id: String,
    pub channel_type: String,
    pub payload_json: String,
    pub retries: u32,
    pub last_error: Option<String>,
    pub next_retry_at: Option<String>,
    pub status: String, // "pending", "dead", "delivered"
    pub created_at: String,
}

/// Enqueue a failed delivery for later retry.
pub async fn enqueue_retry(
    pool: &SqlitePool,
    channel_id: &str,
    channel_type: &str,
    payload: &serde_json::Value,
    error: &str,
) -> Result<()> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let next_retry = (chrono::Utc::now() + chrono::Duration::seconds(10)).to_rfc3339();

    sqlx::query(
        r#"INSERT INTO delivery_queue (id, channel_id, channel_type, payload_json, retries, last_error, next_retry_at, status, created_at)
           VALUES (?, ?, ?, ?, 0, ?, ?, 'pending', ?)"#,
    )
    .bind(&id)
    .bind(channel_id)
    .bind(channel_type)
    .bind(payload.to_string())
    .bind(error)
    .bind(&next_retry)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(())
}

/// Process all pending retries.
/// Call this periodically (e.g. from a background tokio task).
pub async fn process_retry_queue(pool: &SqlitePool) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();

    let pending = sqlx::query_as::<_, QueuedDelivery>(
        "SELECT * FROM delivery_queue WHERE status = 'pending' AND next_retry_at <= ?",
    )
    .bind(&now)
    .fetch_all(pool)
    .await?;

    for item in &pending {
        let payload: serde_json::Value = serde_json::from_str(&item.payload_json)?;
        let success = match item.channel_type.as_str() {
            "webhook" => {
                let url = payload
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if url.is_empty() {
                    false
                } else {
                    let client = reqwest::Client::builder()
                        .timeout(Duration::from_secs(10))
                        .build()
                        .unwrap();
                    match client.post(url).json(&payload).send().await {
                        Ok(r) => r.status().is_success(),
                        Err(_) => false,
                    }
                }
            }
            _ => false,
        };

        if success {
            sqlx::query("UPDATE delivery_queue SET status = 'delivered' WHERE id = ?")
                .bind(&item.id)
                .execute(pool)
                .await?;
        } else {
            let new_retries = item.retries + 1;
            if new_retries >= MAX_RETRIES {
                // Move to dead letter
                sqlx::query(
                    "UPDATE delivery_queue SET status = 'dead', retries = ? WHERE id = ?",
                )
                .bind(new_retries)
                .bind(&item.id)
                .execute(pool)
                .await?;
            } else {
                let backoff = BASE_DELAY_MS * 2u64.pow(new_retries);
                let next =
                    (chrono::Utc::now() + chrono::Duration::milliseconds(backoff as i64))
                        .to_rfc3339();
                sqlx::query(
                    "UPDATE delivery_queue SET retries = ?, last_error = 'retrying', next_retry_at = ? WHERE id = ?",
                )
                .bind(new_retries)
                .bind(&next)
                .bind(&item.id)
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(())
}

/// Start the background retry worker.
pub fn start_retry_worker(pool: SqlitePool) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(30)).await;
            if let Err(e) = process_retry_queue(&pool).await {
                tracing::warn!("retry queue processing error: {}", e);
            }
        }
    });
}
