//! Metrics collector — SSH-based metric gathering.
//!
//! Runs remote commands (`top`, `df`, `free`, `/proc/net/dev`, `/proc/loadavg`)
//! to gather CPU, memory, disk, and network metrics from managed hosts.

use std::sync::Arc;

use chrono::Utc;
use ops_pilot_core::ssh::SshConnectionPool;
use tracing::debug;

use super::models::{HostMetrics, MetricPoint, MetricType};

pub struct MetricsCollector {
    executor: ops_pilot_core::ssh::CommandExecutor,
}

impl MetricsCollector {
    pub fn new(ssh_pool: Arc<SshConnectionPool>) -> Self {
        Self {
            executor: ops_pilot_core::ssh::CommandExecutor::new(ssh_pool),
        }
    }

    /// Collect all metrics from a single host via SSH.
    pub async fn collect_host(&mut self, host_id: &str) -> anyhow::Result<HostMetrics> {
        let cpu_percent = self.collect_cpu(host_id).await.unwrap_or(0.0);
        let (mem_percent, mem_total, mem_used) = self.collect_memory(host_id).await.unwrap_or((0.0, 0.0, 0.0));
        let (disk_percent, disk_total, disk_used) = self.collect_disk(host_id).await.unwrap_or((0.0, 0.0, 0.0));
        let (net_in, net_out) = self.collect_network(host_id).await.unwrap_or((0, 0));
        let (load_1, load_5, load_15) = self.collect_load(host_id).await.unwrap_or((0.0, 0.0, 0.0));

        Ok(HostMetrics {
            host_id: host_id.to_string(),
            timestamp: Utc::now(),
            cpu_percent,
            memory_percent: mem_percent,
            memory_total_mb: mem_total,
            memory_used_mb: mem_used,
            disk_percent,
            disk_total_gb: disk_total,
            disk_used_gb: disk_used,
            network_in_bytes: net_in,
            network_out_bytes: net_out,
            load_1,
            load_5,
            load_15,
        })
    }

    /// Convert HostMetrics to individual MetricPoints for time-series storage.
    pub fn metrics_to_points(&self, metrics: &HostMetrics) -> Vec<MetricPoint> {
        let mut points = Vec::new();
        let ts = metrics.timestamp;
        let hid = &metrics.host_id;

        points.push(MetricPoint {
            timestamp: ts,
            host_id: hid.clone(),
            metric_type: MetricType::CpuUsage,
            value: metrics.cpu_percent,
            unit: "%".into(),
        });
        points.push(MetricPoint {
            timestamp: ts,
            host_id: hid.clone(),
            metric_type: MetricType::MemoryUsage,
            value: metrics.memory_percent,
            unit: "%".into(),
        });
        points.push(MetricPoint {
            timestamp: ts,
            host_id: hid.clone(),
            metric_type: MetricType::DiskUsage,
            value: metrics.disk_percent,
            unit: "%".into(),
        });
        points.push(MetricPoint {
            timestamp: ts,
            host_id: hid.clone(),
            metric_type: MetricType::NetworkIn,
            value: metrics.network_in_bytes as f64,
            unit: "bytes".into(),
        });
        points.push(MetricPoint {
            timestamp: ts,
            host_id: hid.clone(),
            metric_type: MetricType::NetworkOut,
            value: metrics.network_out_bytes as f64,
            unit: "bytes".into(),
        });
        points.push(MetricPoint {
            timestamp: ts,
            host_id: hid.clone(),
            metric_type: MetricType::LoadAverage,
            value: metrics.load_1,
            unit: "1m".into(),
        });

        points
    }

    /// Collect CPU usage via `top -bn1`.
    async fn collect_cpu(&self, host_id: &str) -> anyhow::Result<f64> {
        let result = self
            .executor
            .exec_on_host(host_id, "top -bn1 | head -5")
            .await?;
        if !result.success() {
            anyhow::bail!("top command failed: {}", result.stderr);
        }
        // Parse "Cpu(s): 12.3 us, ..." from top output
        for line in result.stdout.lines() {
            if line.contains("Cpu(s)") || line.contains("cpu ") {
                if let Some(us_pos) = line.find("us") {
                    let before = &line[..us_pos];
                    if let Some(comma_pos) = before.rfind(',') {
                        let val_str = before[comma_pos + 1..].trim();
                        if let Ok(val) = val_str.parse::<f64>() {
                            debug!(host_id, cpu = val, "collected CPU usage");
                            return Ok(val);
                        }
                    }
                }
            }
        }
        Ok(0.0)
    }

    /// Collect memory info via `free -m`.
    async fn collect_memory(&self, host_id: &str) -> anyhow::Result<(f64, f64, f64)> {
        let result = self
            .executor
            .exec_on_host(host_id, "free -m | grep Mem")
            .await?;
        if !result.success() {
            anyhow::bail!("free command failed: {}", result.stderr);
        }
        let parts: Vec<&str> = result.stdout.split_whitespace().collect();
        if parts.len() >= 3 {
            let total = parts[1].parse::<f64>().unwrap_or(0.0);
            let used = parts[2].parse::<f64>().unwrap_or(0.0);
            let percent = if total > 0.0 { used / total * 100.0 } else { 0.0 };
            return Ok((percent, total, used));
        }
        Ok((0.0, 0.0, 0.0))
    }

    /// Collect disk usage via `df -h /`.
    async fn collect_disk(&self, host_id: &str) -> anyhow::Result<(f64, f64, f64)> {
        let result = self
            .executor
            .exec_on_host(host_id, "df -h / | tail -1")
            .await?;
        if !result.success() {
            anyhow::bail!("df command failed: {}", result.stderr);
        }
        let parts: Vec<&str> = result.stdout.split_whitespace().collect();
        if parts.len() >= 5 {
            let total_str = parts[1].trim_end_matches('G').trim_end_matches('M').trim_end_matches('K');
            let used_str = parts[2].trim_end_matches('G').trim_end_matches('M').trim_end_matches('K');
            let total = total_str.parse::<f64>().unwrap_or(0.0);
            let used = used_str.parse::<f64>().unwrap_or(0.0);
            let percent = parts[4].trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
            return Ok((percent, total, used));
        }
        Ok((0.0, 0.0, 0.0))
    }

    /// Collect network I/O from `/proc/net/dev`.
    async fn collect_network(&self, host_id: &str) -> anyhow::Result<(u64, u64)> {
        let result = self
            .executor
            .exec_on_host(host_id, "cat /proc/net/dev | grep eth")
            .await?;
        if !result.success() {
            anyhow::bail!("net dev read failed: {}", result.stderr);
        }
        for line in result.stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("eth") || trimmed.contains(':') {
                if let Some(colon_pos) = trimmed.find(':') {
                    let rest = trimmed[colon_pos + 1..].trim();
                    let fields: Vec<&str> = rest.split_whitespace().collect();
                    if fields.len() >= 10 {
                        let rx_bytes = fields[0].parse::<u64>().unwrap_or(0);
                        let tx_bytes = fields[8].parse::<u64>().unwrap_or(0);
                        return Ok((rx_bytes, tx_bytes));
                    }
                }
            }
        }
        Ok((0, 0))
    }

    /// Collect load average from `/proc/loadavg`.
    async fn collect_load(&self, host_id: &str) -> anyhow::Result<(f64, f64, f64)> {
        let result = self
            .executor
            .exec_on_host(host_id, "cat /proc/loadavg")
            .await?;
        if !result.success() {
            anyhow::bail!("loadavg read failed: {}", result.stderr);
        }
        let parts: Vec<&str> = result.stdout.split_whitespace().collect();
        if parts.len() >= 3 {
            let load_1 = parts[0].parse::<f64>().unwrap_or(0.0);
            let load_5 = parts[1].parse::<f64>().unwrap_or(0.0);
            let load_15 = parts[2].parse::<f64>().unwrap_or(0.0);
            return Ok((load_1, load_5, load_15));
        }
        Ok((0.0, 0.0, 0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_to_points() {
        let collector = MetricsCollector::new(Arc::new(SshConnectionPool::new()));
        let metrics = HostMetrics {
            host_id: "test-host".into(),
            timestamp: Utc::now(),
            cpu_percent: 45.2,
            memory_percent: 67.8,
            memory_total_mb: 8192.0,
            memory_used_mb: 5556.0,
            disk_percent: 72.1,
            disk_total_gb: 100.0,
            disk_used_gb: 72.1,
            network_in_bytes: 123456,
            network_out_bytes: 789012,
            load_1: 1.5,
            load_5: 1.2,
            load_15: 0.8,
        };
        let points = collector.metrics_to_points(&metrics);
        assert_eq!(points.len(), 6);
    }
}
