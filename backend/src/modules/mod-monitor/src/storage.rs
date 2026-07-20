//! Time-series storage for monitoring metrics using SQLite.
//!
//! Metrics are partitioned by host and metric type. A 7-day retention
//! policy automatically purges old data.

use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use tracing::info;

use super::models::{MetricPoint, MetricType};

pub struct MetricStore {
    pool: SqlitePool,
}

impl MetricStore {
    pub async fn new(pool: SqlitePool) -> Self {
        let store = Self { pool };
        let _ = store.ensure_table().await;
        store
    }

    /// Ensure the metrics table exists.
    async fn ensure_table(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS monitor_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                host_id TEXT NOT NULL,
                metric_type TEXT NOT NULL,
                value REAL NOT NULL,
                unit TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_monitor_metrics_host_time
             ON monitor_metrics(host_id, timestamp)",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Insert a single metric data point.
    pub async fn insert_point(&self, point: &MetricPoint) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO monitor_metrics (timestamp, host_id, metric_type, value, unit)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(point.timestamp.to_rfc3339())
        .bind(&point.host_id)
        .bind(serde_json::to_string(&point.metric_type)?)
        .bind(point.value)
        .bind(&point.unit)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Query metric points for a host, optionally filtered by metric type and time range.
    pub async fn query_points(
        &self,
        host_id: &str,
        metric_filter: Option<&str>,
        since: Option<&str>,
    ) -> anyhow::Result<Vec<MetricPoint>> {
        let since_time = since
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now() - Duration::hours(24));

        let rows: Vec<(String, String, String, f64, String)> = if let Some(metric) = metric_filter {
            let metric_type = match metric {
                "cpu" => serde_json::to_string(&MetricType::CpuUsage)?,
                "memory" => serde_json::to_string(&MetricType::MemoryUsage)?,
                "disk" => serde_json::to_string(&MetricType::DiskUsage)?,
                "network" => serde_json::to_string(&MetricType::NetworkIn)?,
                _ => metric.to_string(),
            };
            sqlx::query_as(
                "SELECT timestamp, host_id, metric_type, value, unit
                 FROM monitor_metrics
                 WHERE host_id = ? AND metric_type = ? AND timestamp >= ?
                 ORDER BY timestamp ASC",
            )
            .bind(host_id)
            .bind(&metric_type)
            .bind(since_time.to_rfc3339())
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                "SELECT timestamp, host_id, metric_type, value, unit
                 FROM monitor_metrics
                 WHERE host_id = ? AND timestamp >= ?
                 ORDER BY timestamp ASC",
            )
            .bind(host_id)
            .bind(since_time.to_rfc3339())
            .fetch_all(&self.pool)
            .await?
        };

        let mut points = Vec::new();
        for (ts_str, hid, mt_str, value, unit) in rows {
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&ts_str) {
                let metric_type: MetricType = serde_json::from_str(&mt_str)
                    .unwrap_or(MetricType::CpuUsage);
                points.push(MetricPoint {
                    timestamp: timestamp.with_timezone(&Utc),
                    host_id: hid,
                    metric_type,
                    value,
                    unit,
                });
            }
        }

        Ok(points)
    }

    /// Purge metrics older than the retention period (default 7 days).
    pub async fn purge_old(&self, retention_days: i64) -> anyhow::Result<u64> {
        let cutoff = Utc::now() - Duration::days(retention_days);
        let result = sqlx::query("DELETE FROM monitor_metrics WHERE timestamp < ?")
            .bind(cutoff.to_rfc3339())
            .execute(&self.pool)
            .await?;
        let deleted = result.rows_affected();
        if deleted > 0 {
            info!(deleted, retention_days, "Purged old metrics");
        }
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_store_operations() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let store = MetricStore::new(pool).await;

        let point = MetricPoint {
            timestamp: Utc::now(),
            host_id: "test-host".into(),
            metric_type: MetricType::CpuUsage,
            value: 42.5,
            unit: "%".into(),
        };
        store.insert_point(&point).await.unwrap();

        let points = store.query_points("test-host", Some("cpu"), None).await.unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].value, 42.5);
    }
}
