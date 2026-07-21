//! Prometheus metrics for OpsPilot gateway.
//!
//! Exposes `/api/metrics` endpoint in Prometheus text format.

use axum::{extract::Request, response::Response, routing::get, Json, Router};
use prometheus::{
    register_counter, register_gauge, register_histogram_vec,
    Counter, Encoder, Gauge, HistogramVec, TextEncoder,
};
use std::sync::LazyLock;
use std::time::Instant;

// ── Metrics ──────────────────────────────────────────────────────────────

static HTTP_REQUESTS_TOTAL: LazyLock<Counter> = LazyLock::new(|| {
    register_counter!(
        "ops_pilot_http_requests_total",
        "Total number of HTTP requests"
    )
    .unwrap()
});

static HTTP_REQUEST_DURATION: LazyLock<HistogramVec> = LazyLock::new(|| {
    register_histogram_vec!(
        "ops_pilot_http_request_duration_seconds",
        "HTTP request latency in seconds",
        &["method", "path", "status"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .unwrap()
});

static WS_CONNECTIONS: LazyLock<Gauge> = LazyLock::new(|| {
    register_gauge!("ops_pilot_ws_connections_active", "Active WebSocket connections").unwrap()
});

static ALERTS_TOTAL: LazyLock<Counter> = LazyLock::new(|| {
    register_counter!("ops_pilot_alerts_processed_total", "Total alerts processed").unwrap()
});

// ── Middleware ────────────────────────────────────────────────────────────

/// Axum middleware that records HTTP request count and latency.
pub async fn metrics_middleware(req: Request, next: axum::middleware::Next) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    HTTP_REQUESTS_TOTAL.inc();

    let response = next.run(req).await;

    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();
    HTTP_REQUEST_DURATION
        .with_label_values(&[&method, &path, &status])
        .observe(duration);

    response
}

// ── Handlers ─────────────────────────────────────────────────────────────

/// GET /api/metrics — Prometheus text-format metrics endpoint.
pub async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap_or_default()
}

/// GET /api/metrics/json — JSON-format metrics for frontend dashboards.
pub async fn metrics_json_handler() -> Json<Vec<serde_json::Value>> {
    let metric_families = prometheus::gather();
    let json: Vec<serde_json::Value> = metric_families
        .iter()
        .map(|mf| {
            let name = mf.get_name();
            let help = mf.get_help();
            let proto_metrics = mf.get_metric();
            serde_json::json!({
                "name": name,
                "help": help,
                "metrics": proto_metrics.iter().map(|m| {
                    let mut labels = serde_json::Map::new();
                    for label in m.get_label() {
                        labels.insert(label.get_name().to_string(), serde_json::Value::String(label.get_value().to_string()));
                    }
                    let value = if m.has_counter() { m.get_counter().get_value() }
                        else if m.has_gauge() { m.get_gauge().get_value() }
                        else if m.has_histogram() { m.get_histogram().get_sample_count() as f64 }
                        else { 0.0 };
                    serde_json::json!({ "labels": labels, "value": value })
                }).collect::<Vec<_>>(),
            })
        })
        .collect();
    Json(json)
}

// ── Helpers ──────────────────────────────────────────────────────────────

pub fn inc_alerts() {
    ALERTS_TOTAL.inc();
}

pub fn ws_connected() {
    WS_CONNECTIONS.inc();
}

pub fn ws_disconnected() {
    WS_CONNECTIONS.dec();
}
