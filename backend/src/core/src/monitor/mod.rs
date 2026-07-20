//! Host health monitoring: CPU, memory, and disk usage metrics.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};

/// Aggregated host health stats.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HostStats {
    pub cpu_usage_percent: f64,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub memory_usage_percent: f64,
    pub disk_total_bytes: u64,
    pub disk_used_bytes: u64,
    pub disk_usage_percent: f64,
}

/// Read CPU usage from `/proc/stat` by sampling two snapshots.
///
/// Returns overall CPU usage as a percentage (0.0–100.0).
pub fn read_cpu_usage() -> Result<f64> {
    let sample = read_proc_stat()?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    let sample2 = read_proc_stat()?;

    let idle1 = sample.idle + sample.iowait;
    let idle2 = sample2.idle + sample2.iowait;
    let total1 = sample.total();
    let total2 = sample2.total();

    let idle_diff = idle2.saturating_sub(idle1);
    let total_diff = total2.saturating_sub(total1);

    if total_diff == 0 {
        return Ok(0.0);
    }

    let usage = (1.0 - idle_diff as f64 / total_diff as f64) * 100.0;
    Ok(usage.clamp(0.0, 100.0))
}

#[derive(Debug)]
struct ProcStat {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}

impl ProcStat {
    fn total(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
    }
}

fn read_proc_stat() -> Result<ProcStat> {
    let content = std::fs::read_to_string("/proc/stat").context("failed to read /proc/stat")?;
    let line = content
        .lines()
        .find(|l| l.starts_with("cpu "))
        .context("cpu line not found in /proc/stat")?;

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        anyhow::bail!("unexpected /proc/stat format");
    }

    Ok(ProcStat {
        user: parts[1].parse().context("parse user")?,
        nice: parts[2].parse().context("parse nice")?,
        system: parts[3].parse().context("parse system")?,
        idle: parts[4].parse().context("parse idle")?,
        iowait: parts[5].parse().context("parse iowait")?,
        irq: parts[6].parse().context("parse irq")?,
        softirq: parts[7].parse().context("parse softirq")?,
        steal: parts[8].parse().context("parse steal")?,
    })
}

/// Read memory usage from `/proc/meminfo`.
pub fn read_memory_usage() -> Result<(u64, u64, f64)> {
    let content =
        std::fs::read_to_string("/proc/meminfo").context("failed to read /proc/meminfo")?;

    let mut total = 0u64;
    let mut available = 0u64;

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("MemTotal:") {
            total = parse_mem_kb(val);
        } else if let Some(val) = line.strip_prefix("MemAvailable:") {
            available = parse_mem_kb(val);
        }
    }

    if total == 0 {
        anyhow::bail!("MemTotal not found in /proc/meminfo");
    }

    let used = total.saturating_sub(available);
    let percent = used as f64 / total as f64 * 100.0;
    Ok((total, used, percent.clamp(0.0, 100.0)))
}

fn parse_mem_kb(s: &str) -> u64 {
    s.split_whitespace()
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
        * 1024 // kB → bytes
}

/// Read disk usage via `statvfs` for the given path (default: `/`).
pub fn read_disk_usage(path: &str) -> Result<(u64, u64, f64)> {
    let c_path = std::ffi::CString::new(path).context("invalid path")?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };

    let ret = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if ret != 0 {
        anyhow::bail!(
            "statvfs failed for {}: {}",
            path,
            std::io::Error::last_os_error()
        );
    }

    let block_size = stat.f_frsize;
    let total = stat.f_blocks * block_size;
    let available = stat.f_bavail * block_size;
    let used = total - (stat.f_bfree * block_size - available);

    let percent = if total > 0 {
        used as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    Ok((total, used, percent.clamp(0.0, 100.0)))
}

/// Collect all host stats.
pub fn get_host_stats() -> Result<HostStats> {
    let cpu = read_cpu_usage()?;
    let (mem_total, mem_used, mem_pct) = read_memory_usage()?;
    let (disk_total, disk_used, disk_pct) = read_disk_usage("/")?;

    Ok(HostStats {
        cpu_usage_percent: cpu,
        memory_total_bytes: mem_total,
        memory_used_bytes: mem_used,
        memory_usage_percent: mem_pct,
        disk_total_bytes: disk_total,
        disk_used_bytes: disk_used,
        disk_usage_percent: disk_pct,
    })
}

/// The monitor module implementing `OpsModule`.
#[derive(Default)]
pub struct MonitorModule;

impl MonitorModule {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl OpsModule for MonitorModule {
    fn name(&self) -> &str {
        "monitor"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "Host health monitoring: CPU, memory, and disk usage"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "monitor.get_stats".into(),
            description: "Get current host health stats (CPU, memory, disk usage)".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Disk path to check (default: /)"
                    }
                },
                "required": []
            }),
        }]
    }

    async fn execute(&self, _ctx: &ModuleContext, tool: &str, params: Value) -> Result<Value> {
        match tool {
            "monitor.get_stats" => {
                let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("/");
                let stats = get_host_stats()?;
                let mut result = serde_json::to_value(&stats)?;
                result["disk_path"] = json!(path);
                Ok(result)
            }
            _ => anyhow::bail!("unknown tool: {}", tool),
        }
    }

    async fn on_event(&self, _ctx: &ModuleContext, _event: &OpsEvent) -> Option<ModuleAction> {
        None
    }

    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
        match get_host_stats() {
            Ok(stats) => {
                if stats.cpu_usage_percent > 90.0 || stats.memory_usage_percent > 95.0 {
                    HealthStatus::Unhealthy {
                        reason: format!(
                            "CPU: {:.1}%, Memory: {:.1}%",
                            stats.cpu_usage_percent, stats.memory_usage_percent
                        ),
                    }
                } else if stats.cpu_usage_percent > 70.0 || stats.memory_usage_percent > 85.0 {
                    HealthStatus::Degraded {
                        reason: format!(
                            "CPU: {:.1}%, Memory: {:.1}%",
                            stats.cpu_usage_percent, stats.memory_usage_percent
                        ),
                    }
                } else {
                    HealthStatus::Healthy
                }
            }
            Err(e) => HealthStatus::Unhealthy {
                reason: format!("monitor error: {}", e),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_cpu_usage() {
        let result = read_cpu_usage();
        assert!(
            result.is_ok(),
            "CPU read should succeed: {:?}",
            result.err()
        );
        let usage = result.unwrap();
        assert!(
            (0.0..=100.0).contains(&usage),
            "CPU usage should be 0-100: {}",
            usage
        );
    }

    #[test]
    fn test_read_memory_usage() {
        let result = read_memory_usage();
        assert!(
            result.is_ok(),
            "Memory read should succeed: {:?}",
            result.err()
        );
        let (total, used, pct) = result.unwrap();
        assert!(total > 0, "Total memory should be positive");
        assert!(used <= total, "Used should not exceed total");
        assert!(
            (0.0..=100.0).contains(&pct),
            "Memory percent should be 0-100: {}",
            pct
        );
    }

    #[test]
    fn test_read_disk_usage() {
        let result = read_disk_usage("/");
        assert!(
            result.is_ok(),
            "Disk read should succeed: {:?}",
            result.err()
        );
        let (total, used, pct) = result.unwrap();
        assert!(total > 0, "Total disk should be positive");
        assert!(used <= total, "Used should not exceed total");
        assert!(
            (0.0..=100.0).contains(&pct),
            "Disk percent should be 0-100: {}",
            pct
        );
    }

    #[test]
    fn test_get_host_stats() {
        let stats = get_host_stats();
        assert!(
            stats.is_ok(),
            "get_host_stats should succeed: {:?}",
            stats.err()
        );
        let stats = stats.unwrap();
        assert!((0.0..=100.0).contains(&stats.cpu_usage_percent));
        assert!(stats.memory_total_bytes > 0);
        assert!(stats.disk_total_bytes > 0);
    }

    #[test]
    fn test_monitor_module_metadata() {
        let m = MonitorModule::new();
        assert_eq!(m.name(), "monitor");
        assert_eq!(m.version(), "0.1.0");
        assert!(m.dependencies().is_empty());
        assert_eq!(m.tools().len(), 1);
        assert_eq!(m.tools()[0].name, "monitor.get_stats");
    }

    #[tokio::test]
    async fn test_monitor_execute_get_stats() {
        use ops_pilot_sdk::context::{EventBus, ModuleContext};
        use std::path::PathBuf;
        use std::sync::Arc;

        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let ctx = ModuleContext::new(
            Arc::new(pool),
            EventBus::new(16),
            PathBuf::from("/tmp/monitor-test"),
            "monitor".into(),
        );

        let m = MonitorModule::new();
        let result = m.execute(&ctx, "monitor.get_stats", json!({})).await;
        assert!(result.is_ok(), "execute should succeed: {:?}", result.err());
        let val = result.unwrap();
        assert!(val.get("cpu_usage_percent").is_some());
        assert!(val.get("memory_total_bytes").is_some());
        assert!(val.get("disk_total_bytes").is_some());
    }

    #[tokio::test]
    async fn test_monitor_health_check() {
        use ops_pilot_sdk::context::{EventBus, ModuleContext};
        use std::path::PathBuf;
        use std::sync::Arc;

        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let ctx = ModuleContext::new(
            Arc::new(pool),
            EventBus::new(16),
            PathBuf::from("/tmp/monitor-test"),
            "monitor".into(),
        );

        let m = MonitorModule::new();
        let status = m.health_check(&ctx).await;
        assert!(matches!(
            status,
            HealthStatus::Healthy | HealthStatus::Degraded { .. } | HealthStatus::Unhealthy { .. }
        ));
    }

    #[test]
    fn test_parse_mem_kb() {
        assert_eq!(parse_mem_kb("  16384000 kB"), 16384000 * 1024);
        assert_eq!(parse_mem_kb("0 kB"), 0);
        assert_eq!(parse_mem_kb("bad"), 0);
    }

    #[test]
    fn test_host_stats_serialization() {
        let stats = HostStats {
            cpu_usage_percent: 45.5,
            memory_total_bytes: 16_000_000_000,
            memory_used_bytes: 8_000_000_000,
            memory_usage_percent: 50.0,
            disk_total_bytes: 500_000_000_000,
            disk_used_bytes: 200_000_000_000,
            disk_usage_percent: 40.0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: HostStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, deserialized);
    }
}
