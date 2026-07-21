use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Shared application state for predictions routes.
#[derive(Clone)]
pub struct PredictionsState {
    pub pool: SqlitePool,
}

// ── Request/Response Types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub host_id: String,
    pub metric_type: String,
    pub forecast_hours: Option<u32>,
    pub threshold: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct BatchAnalyzeRequest {
    pub forecast_hours: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct PredictionResult {
    pub host_id: String,
    pub metric_type: String,
    pub current_value: f64,
    pub predicted_value: f64,
    pub trend: String,
    pub confidence: f64,
    pub risk_level: String,
    pub estimated_time_to_threshold_hours: Option<f64>,
    pub data_points: Vec<DataPoint>,
}

#[derive(Debug, Serialize)]
pub struct DataPoint {
    pub timestamp: String,
    pub actual: Option<f64>,
    pub predicted: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RiskItem {
    pub host_id: String,
    pub host_name: String,
    pub metric_type: String,
    pub current_value: f64,
    pub predicted_value: f64,
    pub threshold: f64,
    pub risk_level: String,
    pub estimated_time_hours: Option<f64>,
    pub suggestion: String,
}

// ── Prediction Algorithms ──────────────────────────────────────────────

fn pseudo_random(seed: u64) -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    ((nanos.wrapping_mul(6364136223846793005).wrapping_add(seed)) % 1000) as f64 / 1000.0
}

/// Simple linear regression for time series
fn linear_regression(values: &[f64]) -> (f64, f64) {
    let n = values.len() as f64;
    if n == 0.0 {
        return (0.0, 0.0);
    }

    let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
    let sum_y: f64 = values.iter().sum();
    let sum_xy: f64 = values.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
    let sum_x2: f64 = (0..values.len()).map(|i| (i as f64) * (i as f64)).sum();

    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
    let intercept = (sum_y - slope * sum_x) / n;

    (slope, intercept)
}

/// Exponential smoothing
fn exponential_smoothing(values: &[f64], alpha: f64) -> Vec<f64> {
    if values.is_empty() {
        return vec![];
    }

    let mut result = vec![values[0]];
    for i in 1..values.len() {
        let smoothed = alpha * values[i] + (1.0 - alpha) * result[i - 1];
        result.push(smoothed);
    }
    result
}

/// Generate simulated metric data
fn generate_metric_data(base: f64, trend: f64, points: usize) -> Vec<f64> {
    let mut data = Vec::new();
    for i in 0..points {
        let value = base + trend * (i as f64) + pseudo_random(i as u64) * 10.0 - 5.0;
        data.push(value.max(0.0).min(100.0));
    }
    data
}

/// Analyze a metric and predict future values
fn analyze_metric(
    host_id: &str,
    metric_type: &str,
    forecast_hours: u32,
    threshold: f64,
) -> PredictionResult {
    // Generate historical data points (last 24 hours, hourly)
    let historical_points = 24;
    let (base, trend) = match metric_type {
        "cpu" => (45.0, 0.5),
        "memory" => (60.0, 0.3),
        "disk" => (70.0, 0.2),
        "network" => (30.0, -0.1),
        _ => (50.0, 0.0),
    };

    let historical = generate_metric_data(base, trend, historical_points);

    // Linear regression
    let (slope, intercept) = linear_regression(&historical);

    // Predict future values
    let mut data_points: Vec<DataPoint> = Vec::new();
    let now = chrono::Utc::now();

    // Historical points
    for (i, value) in historical.iter().enumerate() {
        let ts = now - chrono::Duration::hours((historical_points - i) as i64);
        data_points.push(DataPoint {
            timestamp: ts.to_rfc3339(),
            actual: Some(*value),
            predicted: None,
        });
    }

    // Future predictions
    let mut predicted_values = Vec::new();
    for i in 0..forecast_hours {
        let predicted = intercept + slope * (historical_points as f64 + i as f64);
        let predicted = predicted.max(0.0).min(100.0);
        predicted_values.push(predicted);

        let ts = now + chrono::Duration::hours(i as i64);
        data_points.push(DataPoint {
            timestamp: ts.to_rfc3339(),
            actual: None,
            predicted: Some(predicted),
        });
    }

    let current_value = historical.last().copied().unwrap_or(0.0);
    let predicted_value = predicted_values.last().copied().unwrap_or(current_value);

    // Determine trend
    let trend_str = if slope > 0.5 {
        "up"
    } else if slope < -0.5 {
        "down"
    } else {
        "stable"
    };

    // Calculate confidence (based on how linear the data is)
    let smoothed = exponential_smoothing(&historical, 0.3);
    let residuals: Vec<f64> = historical
        .iter()
        .zip(smoothed.iter())
        .map(|(a, s)| (a - s).abs())
        .collect();
    let avg_residual = residuals.iter().sum::<f64>() / residuals.len() as f64;
    let confidence = (1.0 - avg_residual / 50.0).max(0.3).min(0.95);

    // Risk assessment
    let (risk_level, est_time) = if predicted_value >= threshold {
        ("critical".to_string(), Some(0.0))
    } else if predicted_value >= threshold * 0.9 {
        let hours_to_threshold = if slope > 0.0 {
            ((threshold - current_value) / slope).max(0.0)
        } else {
            f64::INFINITY
        };
        if hours_to_threshold < forecast_hours as f64 {
            ("warning".to_string(), Some(hours_to_threshold))
        } else {
            ("safe".to_string(), None)
        }
    } else {
        ("safe".to_string(), None)
    };

    PredictionResult {
        host_id: host_id.to_string(),
        metric_type: metric_type.to_string(),
        current_value,
        predicted_value,
        trend: trend_str.to_string(),
        confidence,
        risk_level,
        estimated_time_to_threshold_hours: est_time,
        data_points,
    }
}

// ── Handlers ───────────────────────────────────────────────────────────

/// POST /api/predictions/analyze — analyze a metric prediction
pub async fn analyze_prediction(
    State(_state): State<PredictionsState>,
    Json(req): Json<AnalyzeRequest>,
) -> impl IntoResponse {
    let forecast_hours = req.forecast_hours.unwrap_or(24);
    let threshold = req.threshold.unwrap_or(80.0);

    let result = analyze_metric(&req.host_id, &req.metric_type, forecast_hours, threshold);

    (StatusCode::OK, Json(result)).into_response()
}

/// POST /api/predictions/batch — batch prediction for all hosts
pub async fn batch_prediction(
    State(_state): State<PredictionsState>,
    Json(req): Json<BatchAnalyzeRequest>,
) -> impl IntoResponse {
    let forecast_hours = req.forecast_hours.unwrap_or(24);

    let hosts = vec![
        ("host-001", "web-server"),
        ("host-002", "db-master"),
        ("host-003", "cache-redis"),
    ];

    let metrics = vec![
        ("cpu", 80.0),
        ("memory", 85.0),
        ("disk", 90.0),
        ("network", 100.0),
    ];

    let mut results = Vec::new();
    for (host_id, _host_name) in &hosts {
        for (metric, threshold) in &metrics {
            let result = analyze_metric(host_id, metric, forecast_hours, *threshold);
            results.push(result);
        }
    }

    (StatusCode::OK, Json(results)).into_response()
}

/// GET /api/predictions/risks — list high-risk predictions
pub async fn list_risks(
    State(_state): State<PredictionsState>,
) -> impl IntoResponse {
    let hosts = vec![
        ("host-001", "web-server-01"),
        ("host-002", "db-master"),
        ("host-003", "cache-redis"),
    ];

    let mut risks = Vec::new();
    for (host_id, host_name) in &hosts {
        // Check disk usage
        let disk_value = 70.0 + pseudo_random(host_id.len() as u64) * 25.0;
        if disk_value > 85.0 {
            let est_hours = ((90.0 - disk_value) / 0.2).max(0.0);
            risks.push(RiskItem {
                host_id: host_id.to_string(),
                host_name: host_name.to_string(),
                metric_type: "disk".to_string(),
                current_value: disk_value,
                predicted_value: disk_value + 0.2 * 24.0,
                threshold: 90.0,
                risk_level: if disk_value > 90.0 { "critical".to_string() } else { "warning".to_string() },
                estimated_time_to_threshold_hours: Some(est_hours),
                suggestion: "Consider expanding disk space or cleaning up old files".to_string(),
            });
        }

        // Check memory usage
        let mem_value = 60.0 + pseudo_random(host_id.len() as u64 + 100) * 30.0;
        if mem_value > 80.0 {
            let est_hours = ((95.0 - mem_value) / 0.3).max(0.0);
            risks.push(RiskItem {
                host_id: host_id.to_string(),
                host_name: host_name.to_string(),
                metric_type: "memory".to_string(),
                current_value: mem_value,
                predicted_value: mem_value + 0.3 * 24.0,
                threshold: 95.0,
                risk_level: if mem_value > 90.0 { "critical".to_string() } else { "warning".to_string() },
                estimated_time_to_threshold_hours: Some(est_hours),
                suggestion: "Check for memory leaks or consider adding more RAM".to_string(),
            });
        }
    }

    // Sort by risk level (critical first)
    risks.sort_by(|a, b| {
        let order = |s: &str| match s.as_str() {
            "critical" => 0,
            "warning" => 1,
            _ => 2,
        };
        order(&a.risk_level).cmp(&order(&b.risk_level))
    });

    (StatusCode::OK, Json(risks)).into_response()
}

/// Build the predictions routes sub-router.
pub fn predictions_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post};

    let state = PredictionsState { pool };

    Router::new()
        .route("/api/predictions/analyze", post(analyze_prediction))
        .route("/api/predictions/batch", post(batch_prediction))
        .route("/api/predictions/risks", get(list_risks))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_regression() {
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let (slope, _intercept) = linear_regression(&values);
        assert!((slope - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_exponential_smoothing() {
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let smoothed = exponential_smoothing(&values, 0.5);
        assert_eq!(smoothed.len(), values.len());
        assert!((smoothed[0] - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_analyze_metric() {
        let result = analyze_metric("host-001", "cpu", 24, 80.0);
        assert_eq!(result.host_id, "host-001");
        assert_eq!(result.metric_type, "cpu");
        assert!(result.confidence > 0.0);
    }
}
