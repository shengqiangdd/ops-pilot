//! Docker container management via bollard.
//!
//! Provides [`DockerClient`] for listing, starting, stopping, restarting
//! containers and collecting resource statistics.

use bollard::container::{
    ListContainersOptions, RestartContainerOptions, StartContainerOptions, StatsOptions,
    StopContainerOptions,
};
use bollard::Docker;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur when interacting with the Docker daemon.
#[derive(Debug, Error)]
pub enum DockerError {
    /// The underlying bollard client returned an error.
    #[error("docker API error: {0}")]
    Bollard(#[from] bollard::errors::Error),

    /// The requested container was not found.
    #[error("container not found: {0}")]
    NotFound(String),

    /// A required field was missing from the Docker API response.
    #[error("missing field in Docker response: {0}")]
    MissingField(String),

    /// Stats data was incomplete or could not be parsed.
    #[error("invalid stats data: {0}")]
    InvalidStats(String),
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A running or stopped container known to the Docker daemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Container {
    /// Container ID (short SHA).
    pub id: String,
    /// Container name (without leading `/`).
    pub name: String,
    /// Image name (e.g. `nginx:latest`).
    pub image: String,
    /// Human-readable status string (e.g. "Up 2 hours").
    pub status: String,
    /// Low-level state (`running`, `exited`, …).
    pub state: String,
    /// Unix timestamp (seconds) when the container was created.
    pub created: i64,
}

/// Resource usage statistics for a single container.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContainerStats {
    /// CPU usage as a percentage of available CPU.
    pub cpu_percent: f64,
    /// Memory statistics.
    pub memory: MemoryStats,
    /// Network I/O statistics.
    pub network: NetworkStats,
}

/// Memory usage breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryStats {
    /// Current memory usage in bytes.
    pub usage_bytes: u64,
    /// Memory limit in bytes.
    pub limit_bytes: u64,
    /// Usage as a percentage of the limit.
    pub usage_percent: f64,
}

/// Network I/O counters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkStats {
    /// Total bytes received.
    pub rx_bytes: u64,
    /// Total bytes transmitted.
    pub tx_bytes: u64,
}

// ---------------------------------------------------------------------------
// DockerClient
// ---------------------------------------------------------------------------

/// Client for interacting with the Docker daemon.
///
/// Wraps [`bollard::Docker`] and exposes a focused API used by OpsPilot
/// modules and the core engine.
pub struct DockerClient {
    docker: Docker,
}

impl DockerClient {
    /// Connect to the Docker daemon using the default socket / environment.
    pub async fn connect() -> Result<Self, DockerError> {
        let docker = Docker::connect_with_local_defaults()?;
        Ok(Self { docker })
    }

    /// Connect to a specific Docker host URL.
    pub async fn connect_with_host(host: &str) -> Result<Self, DockerError> {
        let docker = Docker::connect_with_http(host, 120, bollard::API_DEFAULT_VERSION)?;
        Ok(Self { docker })
    }

    /// Create a client from an existing [`bollard::Docker`] handle (useful for
    /// testing with a mock or custom transport).
    pub fn new(docker: Docker) -> Self {
        Self { docker }
    }

    /// Return a reference to the underlying bollard client.
    pub fn inner(&self) -> &Docker {
        &self.docker
    }

    // -- Container operations -------------------------------------------------

    /// List all running containers.
    pub async fn list_containers(&self) -> Result<Vec<Container>, DockerError> {
        let mut filters = HashMap::new();
        filters.insert("status", vec!["running"]);

        let options = Some(ListContainersOptions {
            all: false,
            filters,
            ..Default::default()
        });

        let summaries = self.docker.list_containers(options).await?;

        summaries
            .into_iter()
            .map(|c| {
                let id = c.id.ok_or_else(|| DockerError::MissingField("id".into()))?;
                let names = c.names.ok_or_else(|| DockerError::MissingField("names".into()))?;
                let name = names
                    .into_iter()
                    .next()
                    .unwrap_or_default()
                    .trim_start_matches('/')
                    .to_string();

                let image = c.image.unwrap_or_default();
                let status = c.status.unwrap_or_default();
                let state = c.state.unwrap_or_default();
                let created = c.created.unwrap_or(0);

                Ok(Container {
                    id,
                    name,
                    image,
                    status,
                    state,
                    created,
                })
            })
            .collect()
    }

    /// Start a container by ID.
    pub async fn start_container(&self, id: &str) -> Result<(), DockerError> {
        self.docker
            .start_container(id, None::<StartContainerOptions<String>>)
            .await?;
        Ok(())
    }

    /// Stop a container by ID. Uses the default 10-second timeout.
    pub async fn stop_container(&self, id: &str) -> Result<(), DockerError> {
        let options = Some(StopContainerOptions { t: 10 });
        self.docker.stop_container(id, options).await?;
        Ok(())
    }

    /// Restart a container by ID. Uses the default 10-second timeout.
    pub async fn restart_container(&self, id: &str) -> Result<(), DockerError> {
        let options = Some(RestartContainerOptions { t: 10 });
        self.docker.restart_container(id, options).await?;
        Ok(())
    }

    /// Collect resource-usage statistics for a container.
    ///
    /// The first stats payload from the Docker daemon is consumed and
    /// returned. Call this method each time you need a fresh snapshot.
    pub async fn container_stats(&self, id: &str) -> Result<ContainerStats, DockerError> {
        use futures_util::StreamExt;

        let options = Some(StatsOptions {
            stream: false,
            one_shot: true,
        });

        let mut stream = self.docker.stats(id, options);
        let stats = stream
            .next()
            .await
            .ok_or_else(|| DockerError::InvalidStats("empty stats stream".into()))??;

        Self::parse_stats(&stats)
    }

    /// Parse a bollard `Stats` response into our [`ContainerStats`].
    fn parse_stats(stats: &bollard::container::Stats) -> Result<ContainerStats, DockerError> {
        // CPU ----------------------------------------------------------
        let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
            - stats.precpu_stats.cpu_usage.total_usage as f64;
        let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
            - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
        let num_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;

        let cpu_percent = if system_delta > 0.0 && cpu_delta >= 0.0 {
            (cpu_delta / system_delta) * num_cpus * 100.0
        } else {
            0.0
        };

        // Memory -------------------------------------------------------
        let usage_bytes = stats.memory_stats.usage.unwrap_or(0);
        let limit_bytes = stats.memory_stats.limit.unwrap_or(1);
        let usage_percent = if limit_bytes > 0 {
            (usage_bytes as f64 / limit_bytes as f64) * 100.0
        } else {
            0.0
        };

        // Network ------------------------------------------------------
        let (mut rx_bytes, mut tx_bytes) = (0u64, 0u64);
        if let Some(networks) = &stats.networks {
            for iface in networks.values() {
                rx_bytes += iface.rx_bytes;
                tx_bytes += iface.tx_bytes;
            }
        }

        Ok(ContainerStats {
            cpu_percent,
            memory: MemoryStats {
                usage_bytes,
                limit_bytes,
                usage_percent,
            },
            network: NetworkStats { rx_bytes, tx_bytes },
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Error type tests -------------------------------------------------

    #[test]
    fn docker_error_display_bollard() {
        // Verify the error message format for the Bollard variant.
        let err = DockerError::NotFound("abc123".into());
        assert_eq!(err.to_string(), "container not found: abc123");
    }

    #[test]
    fn docker_error_not_found_display() {
        let err = DockerError::NotFound("deadbeef".into());
        assert!(err.to_string().contains("deadbeef"));
    }

    #[test]
    fn docker_error_missing_field_display() {
        let err = DockerError::MissingField("id".into());
        assert!(err.to_string().contains("missing field"));
        assert!(err.to_string().contains("id"));
    }

    #[test]
    fn docker_error_invalid_stats_display() {
        let err = DockerError::InvalidStats("empty stream".into());
        assert!(err.to_string().contains("invalid stats"));
    }

    #[test]
    fn docker_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DockerError>();
    }

    // -- Data type tests --------------------------------------------------

    #[test]
    fn container_serialize_roundtrip() {
        let c = Container {
            id: "abc123".into(),
            name: "web".into(),
            image: "nginx:latest".into(),
            status: "Up 2 hours".into(),
            state: "running".into(),
            created: 1700000000,
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: Container = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn container_stats_serialize_roundtrip() {
        let stats = ContainerStats {
            cpu_percent: 12.5,
            memory: MemoryStats {
                usage_bytes: 1024 * 1024,
                limit_bytes: 512 * 1024 * 1024,
                usage_percent: 0.2,
            },
            network: NetworkStats {
                rx_bytes: 1000,
                tx_bytes: 500,
            },
        };
        let json = serde_json::to_string(&stats).unwrap();
        let back: ContainerStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, back);
    }

    #[test]
    fn memory_stats_defaults_to_zero_percent_on_zero_limit() {
        let m = MemoryStats {
            usage_bytes: 100,
            limit_bytes: 0,
            usage_percent: 0.0,
        };
        assert_eq!(m.usage_percent, 0.0);
    }

    #[test]
    fn network_stats_addition() {
        let n1 = NetworkStats {
            rx_bytes: 100,
            tx_bytes: 200,
        };
        let n2 = NetworkStats {
            rx_bytes: 50,
            tx_bytes: 75,
        };
        let total = NetworkStats {
            rx_bytes: n1.rx_bytes + n2.rx_bytes,
            tx_bytes: n1.tx_bytes + n2.tx_bytes,
        };
        assert_eq!(total.rx_bytes, 150);
        assert_eq!(total.tx_bytes, 275);
    }

    // -- Client construction tests ----------------------------------------

    #[test]
    fn new_from_bollard_instance() {
        // We can construct a DockerClient from a bollard::Docker handle
        // without needing a live daemon. This verifies the constructor
        // is available and the struct is Send + Sync.
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DockerClient>();
    }

    // -- Stats parsing tests ----------------------------------------------

    /// Build a fully-specified `bollard::container::Stats` with sensible defaults,
    /// since `Stats` does not implement `Default`.
    fn make_stats() -> bollard::container::Stats {
        bollard::container::Stats {
            read: String::new(),
            preread: String::new(),
            num_procs: 0,
            pids_stats: bollard::container::PidsStats {
                current: None,
                limit: None,
            },
            network: None,
            networks: None,
            memory_stats: bollard::container::MemoryStats {
                stats: None,
                max_usage: Some(0),
                usage: Some(0),
                failcnt: None,
                limit: Some(1),
                commit: None,
                commit_peak: None,
                commitbytes: None,
                commitpeakbytes: None,
                privateworkingset: None,
            },
            blkio_stats: bollard::container::BlkioStats {
                io_service_bytes_recursive: None,
                io_serviced_recursive: None,
                io_queue_recursive: None,
                io_service_time_recursive: None,
                io_wait_time_recursive: None,
                io_merged_recursive: None,
                io_time_recursive: None,
                sectors_recursive: None,
            },
            cpu_stats: bollard::container::CPUStats {
                cpu_usage: bollard::container::CPUUsage {
                    percpu_usage: None,
                    usage_in_usermode: 0,
                    total_usage: 0,
                    usage_in_kernelmode: 0,
                },
                system_cpu_usage: Some(0),
                online_cpus: Some(1),
                throttling_data: bollard::container::ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            precpu_stats: bollard::container::CPUStats {
                cpu_usage: bollard::container::CPUUsage {
                    percpu_usage: None,
                    usage_in_usermode: 0,
                    total_usage: 0,
                    usage_in_kernelmode: 0,
                },
                system_cpu_usage: Some(0),
                online_cpus: Some(1),
                throttling_data: bollard::container::ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            storage_stats: bollard::container::StorageStats {
                read_count_normalized: None,
                read_size_bytes: None,
                write_count_normalized: None,
                write_size_bytes: None,
            },
            name: String::new(),
            id: String::new(),
        }
    }

    #[test]
    fn parse_stats_with_zero_deltas() {
        let stats = bollard::container::Stats {
            cpu_stats: bollard::container::CPUStats {
                cpu_usage: bollard::container::CPUUsage {
                    total_usage: 100,
                    percpu_usage: None,
                    usage_in_kernelmode: 0,
                    usage_in_usermode: 0,
                },
                system_cpu_usage: Some(1000),
                online_cpus: Some(2),
                throttling_data: bollard::container::ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            precpu_stats: bollard::container::CPUStats {
                cpu_usage: bollard::container::CPUUsage {
                    total_usage: 100,
                    percpu_usage: None,
                    usage_in_kernelmode: 0,
                    usage_in_usermode: 0,
                },
                system_cpu_usage: Some(1000),
                online_cpus: Some(2),
                throttling_data: bollard::container::ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            memory_stats: bollard::container::MemoryStats {
                usage: Some(50_000_000),
                max_usage: Some(100_000_000),
                stats: None,
                failcnt: None,
                limit: Some(512_000_000),
                commit: None,
                commit_peak: None,
                commitbytes: None,
                commitpeakbytes: None,
                privateworkingset: None,
            },
            networks: None,
            ..make_stats()
        };

        let result = DockerClient::parse_stats(&stats).unwrap();
        assert_eq!(result.cpu_percent, 0.0);
        assert_eq!(result.memory.usage_bytes, 50_000_000);
        assert_eq!(result.memory.limit_bytes, 512_000_000);
        assert!((result.memory.usage_percent - 9.765625).abs() < 0.001);
        assert_eq!(result.network.rx_bytes, 0);
        assert_eq!(result.network.tx_bytes, 0);
    }

    #[test]
    fn parse_stats_with_cpu_increase() {
        let stats = bollard::container::Stats {
            cpu_stats: bollard::container::CPUStats {
                cpu_usage: bollard::container::CPUUsage {
                    total_usage: 200,
                    percpu_usage: None,
                    usage_in_kernelmode: 0,
                    usage_in_usermode: 0,
                },
                system_cpu_usage: Some(2000),
                online_cpus: Some(4),
                throttling_data: bollard::container::ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            precpu_stats: bollard::container::CPUStats {
                cpu_usage: bollard::container::CPUUsage {
                    total_usage: 100,
                    percpu_usage: None,
                    usage_in_kernelmode: 0,
                    usage_in_usermode: 0,
                },
                system_cpu_usage: Some(1000),
                online_cpus: Some(4),
                throttling_data: bollard::container::ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            memory_stats: bollard::container::MemoryStats {
                usage: Some(100_000_000),
                max_usage: Some(200_000_000),
                stats: None,
                failcnt: None,
                limit: Some(1_000_000_000),
                commit: None,
                commit_peak: None,
                commitbytes: None,
                commitpeakbytes: None,
                privateworkingset: None,
            },
            networks: None,
            ..make_stats()
        };

        let result = DockerClient::parse_stats(&stats).unwrap();
        // cpu_delta=100, system_delta=1000, num_cpus=4 → (100/1000)*4*100 = 40%
        assert!((result.cpu_percent - 40.0).abs() < 0.001);
    }

    #[test]
    fn parse_stats_with_network_interfaces() {
        let mut networks = HashMap::new();
        networks.insert(
            "eth0".to_string(),
            bollard::container::NetworkStats {
                rx_bytes: 1000,
                rx_packets: 10,
                rx_errors: 0,
                rx_dropped: 0,
                tx_bytes: 500,
                tx_packets: 5,
                tx_errors: 0,
                tx_dropped: 0,
            },
        );
        networks.insert(
            "eth1".to_string(),
            bollard::container::NetworkStats {
                rx_bytes: 2000,
                rx_packets: 20,
                rx_errors: 0,
                rx_dropped: 0,
                tx_bytes: 800,
                tx_packets: 8,
                tx_errors: 0,
                tx_dropped: 0,
            },
        );

        let stats = bollard::container::Stats {
            cpu_stats: bollard::container::CPUStats {
                cpu_usage: bollard::container::CPUUsage {
                    total_usage: 0,
                    percpu_usage: None,
                    usage_in_kernelmode: 0,
                    usage_in_usermode: 0,
                },
                system_cpu_usage: Some(0),
                online_cpus: Some(1),
                throttling_data: bollard::container::ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            precpu_stats: bollard::container::CPUStats {
                cpu_usage: bollard::container::CPUUsage {
                    total_usage: 0,
                    percpu_usage: None,
                    usage_in_kernelmode: 0,
                    usage_in_usermode: 0,
                },
                system_cpu_usage: Some(0),
                online_cpus: Some(1),
                throttling_data: bollard::container::ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            memory_stats: bollard::container::MemoryStats {
                usage: Some(0),
                max_usage: Some(0),
                stats: None,
                failcnt: None,
                limit: Some(1),
                commit: None,
                commit_peak: None,
                commitbytes: None,
                commitpeakbytes: None,
                privateworkingset: None,
            },
            networks: Some(networks),
            ..make_stats()
        };

        let result = DockerClient::parse_stats(&stats).unwrap();
        assert_eq!(result.network.rx_bytes, 3000);
        assert_eq!(result.network.tx_bytes, 1300);
    }

    #[test]
    fn parse_stats_with_no_system_cpu() {
        let stats = bollard::container::Stats {
            cpu_stats: bollard::container::CPUStats {
                cpu_usage: bollard::container::CPUUsage {
                    total_usage: 500,
                    percpu_usage: None,
                    usage_in_kernelmode: 0,
                    usage_in_usermode: 0,
                },
                system_cpu_usage: None,
                online_cpus: None,
                throttling_data: bollard::container::ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            precpu_stats: bollard::container::CPUStats {
                cpu_usage: bollard::container::CPUUsage {
                    total_usage: 500,
                    percpu_usage: None,
                    usage_in_kernelmode: 0,
                    usage_in_usermode: 0,
                },
                system_cpu_usage: None,
                online_cpus: None,
                throttling_data: bollard::container::ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            memory_stats: bollard::container::MemoryStats {
                usage: Some(1024),
                max_usage: Some(2048),
                stats: None,
                failcnt: None,
                limit: Some(4096),
                commit: None,
                commit_peak: None,
                commitbytes: None,
                commitpeakbytes: None,
                privateworkingset: None,
            },
            networks: None,
            ..make_stats()
        };

        let result = DockerClient::parse_stats(&stats).unwrap();
        // Both system_cpu_usage are None → 0 - 0 = 0, so cpu_percent = 0
        assert_eq!(result.cpu_percent, 0.0);
        assert_eq!(result.memory.usage_bytes, 1024);
        assert_eq!(result.memory.limit_bytes, 4096);
    }

    #[test]
    fn parse_stats_memory_zero_limit() {
        let stats = bollard::container::Stats {
            cpu_stats: bollard::container::CPUStats {
                cpu_usage: bollard::container::CPUUsage {
                    total_usage: 0,
                    percpu_usage: None,
                    usage_in_kernelmode: 0,
                    usage_in_usermode: 0,
                },
                system_cpu_usage: Some(0),
                online_cpus: Some(1),
                throttling_data: bollard::container::ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            precpu_stats: bollard::container::CPUStats {
                cpu_usage: bollard::container::CPUUsage {
                    total_usage: 0,
                    percpu_usage: None,
                    usage_in_kernelmode: 0,
                    usage_in_usermode: 0,
                },
                system_cpu_usage: Some(0),
                online_cpus: Some(1),
                throttling_data: bollard::container::ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            memory_stats: bollard::container::MemoryStats {
                usage: Some(100),
                max_usage: Some(200),
                stats: None,
                failcnt: None,
                limit: Some(0),
                commit: None,
                commit_peak: None,
                commitbytes: None,
                commitpeakbytes: None,
                privateworkingset: None,
            },
            networks: None,
            ..make_stats()
        };

        let result = DockerClient::parse_stats(&stats).unwrap();
        assert_eq!(result.memory.usage_percent, 0.0);
    }

    // -- Container struct field tests -------------------------------------

    #[test]
    fn container_fields_preserved() {
        let c = Container {
            id: "sha256:abcdef123456".into(),
            name: "my-app".into(),
            image: "node:20-alpine".into(),
            status: "Up 5 minutes".into(),
            state: "running".into(),
            created: 1_700_000_000,
        };

        assert_eq!(c.id, "sha256:abcdef123456");
        assert_eq!(c.name, "my-app");
        assert_eq!(c.image, "node:20-alpine");
        assert_eq!(c.status, "Up 5 minutes");
        assert_eq!(c.state, "running");
        assert_eq!(c.created, 1_700_000_000);
    }

    #[test]
    fn container_debug_output() {
        let c = Container {
            id: "abc".into(),
            name: "test".into(),
            image: "alpine".into(),
            status: "Up".into(),
            state: "running".into(),
            created: 0,
        };
        let dbg = format!("{:?}", c);
        assert!(dbg.contains("Container"));
        assert!(dbg.contains("abc"));
    }
}
