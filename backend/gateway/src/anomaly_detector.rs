//! 异常检测 —— 基于统计方法的异常点检测。

use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct DetectRequest {
    pub metric: String,
    pub values: Vec<f64>,
}

#[derive(Debug, Serialize)]
pub struct DetectResult {
    pub metric: String,
    pub method: String,
    pub anomalies: Vec<AnomalyPoint>,
    pub stats: DataStats,
}

#[derive(Debug, Serialize)]
pub struct AnomalyPoint {
    pub index: usize,
    pub value: f64,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct DataStats {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
    pub q1: f64,
    pub q3: f64,
    pub iqr: f64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AlertTrend {
    pub date: String,
    pub count: i64,
    pub critical: i64,
    pub warning: i64,
}

pub struct AnomalyDetector {
    pool: SqlitePool,
}

impl AnomalyDetector {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 检测异常点（3-sigma 方法）。
    pub async fn detect(&self, req: &DetectRequest) -> Result<DetectResult, String> {
        if req.values.is_empty() {
            return Err("values cannot be empty".into());
        }

        let stats = compute_stats(&req.values);
        let mut anomalies = Vec::new();

        // 3-sigma 检测
        if stats.std_dev > 0.0 {
            let upper = stats.mean + 3.0 * stats.std_dev;
            let lower = stats.mean - 3.0 * stats.std_dev;
            for (i, &v) in req.values.iter().enumerate() {
                if v > upper || v < lower {
                    anomalies.push(AnomalyPoint {
                        index: i,
                        value: v,
                        reason: if v > upper {
                            format!("超过 3σ 上界 ({:.2})", upper)
                        } else {
                            format!("低于 3σ 下界 ({:.2})", lower)
                        },
                    });
                }
            }
        }

        // IQR 补充检测
        if stats.iqr > 0.0 {
            let lower_fence = stats.q1 - 1.5 * stats.iqr;
            let upper_fence = stats.q3 + 1.5 * stats.iqr;
            for (i, &v) in req.values.iter().enumerate() {
                if (v > upper_fence || v < lower_fence)
                    && !anomalies.iter().any(|a| a.index == i)
                {
                    anomalies.push(AnomalyPoint {
                        index: i,
                        value: v,
                        reason: if v > upper_fence {
                            format!("超过 IQR 上界 ({:.2})", upper_fence)
                        } else {
                            format!("低于 IQR 下界 ({:.2})", lower_fence)
                        },
                    });
                }
            }
        }

        Ok(DetectResult {
            metric: req.metric.clone(),
            method: "3-sigma + IQR".into(),
            anomalies,
            stats,
        })
    }

    /// 分析历史告警趋势。
    pub async fn alert_trends(&self, days: i64) -> Result<Vec<AlertTrend>, String> {
        sqlx::query_as::<_, AlertTrend>(
            "SELECT DATE(created_at) as date, \
             COUNT(*) as count, \
             SUM(CASE WHEN severity = 'critical' THEN 1 ELSE 0 END) as critical, \
             SUM(CASE WHEN severity = 'warning' THEN 1 ELSE 0 END) as warning \
             FROM alert_history \
             WHERE created_at >= datetime('now', '-' || ? || ' days') \
             GROUP BY DATE(created_at) \
             ORDER BY date ASC",
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
    }
}

fn compute_stats(values: &[f64]) -> DataStats {
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let std_dev = variance.sqrt();

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min = sorted[0];
    let max = sorted[n - 1];

    let q1 = percentile(&sorted, 25.0);
    let q3 = percentile(&sorted, 75.0);
    let iqr = q3 - q1;

    DataStats {
        mean,
        std_dev,
        min,
        max,
        count: n,
        q1,
        q3,
        iqr,
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    let k = (p / 100.0) * (n - 1) as f64;
    let f = k.floor() as usize;
    let c = k.ceil() as usize;
    if f == c {
        sorted[f]
    } else {
        sorted[f] * (c as f64 - k) + sorted[c] * (k - f as f64)
    }
}
