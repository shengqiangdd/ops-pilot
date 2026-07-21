//! Data models for monitoring metrics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single metric data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: DateTime<Utc>,
    pub host_id: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
}

/// Type of metric being collected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    CpuUsage,
    MemoryUsage,
    DiskUsage,
    NetworkIn,
    NetworkOut,
    LoadAverage,
}

/// Snapshot of all metrics for a host at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMetrics {
    pub host_id: String,
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub memory_total_mb: f64,
    pub memory_used_mb: f64,
    pub disk_percent: f64,
    pub disk_total_gb: f64,
    pub disk_used_gb: f64,
    pub network_in_bytes: u64,
    pub network_out_bytes: u64,
    pub load_1: f64,
    pub load_5: f64,
    pub load_15: f64,
}
