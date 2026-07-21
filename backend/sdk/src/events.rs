use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use uuid::Uuid;

use crate::context::EventBus;

/// Global event bus singleton. Any crate can publish/subscribe without
/// explicitly passing a bus instance.
static GLOBAL_EVENT_BUS: OnceLock<EventBus> = OnceLock::new();

/// Return a reference to the global event bus (lazily initialized, capacity 1024).
pub fn global_event_bus() -> &'static EventBus {
    GLOBAL_EVENT_BUS.get_or_init(|| EventBus::new(1024))
}

/// Events emitted by core systems and modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpsEvent {
    // ── Core infrastructure events ──────────────────────────────────────
    /// A new connection was added (SSH host, Docker host, etc.).
    ConnectionAdded {
        id: Uuid,
        kind: String,
        name: String,
    },
    /// A connection was removed.
    ConnectionRemoved { id: Uuid, kind: String },
    /// A remote command was executed.
    CommandExecuted {
        host_id: Uuid,
        command: String,
        exit_code: i32,
    },
    /// A Docker event occurred.
    DockerEvent {
        container_id: String,
        action: String,
        actor: String,
    },
    /// A health check completed.
    HealthCheck {
        host_id: Uuid,
        status: String,
        details: serde_json::Value,
    },
    /// An audit log entry was recorded.
    AuditLog {
        user: String,
        action: String,
        resource: String,
        outcome: String,
    },
    /// An action performed by a module.
    ModuleAction {
        module: String,
        action: String,
        payload: serde_json::Value,
    },

    // ── Legacy / fine-grained events ────────────────────────────────────
    /// A host health metric changed.
    MetricUpdated {
        host_id: Uuid,
        metric: String,
        value: f64,
    },
    /// A container state changed.
    ContainerStateChanged { container_id: String, state: String },
    /// An alert was triggered.
    AlertTriggered { severity: String, message: String },
    /// A new SSH session was established.
    SshSessionOpened { host_id: Uuid, session_id: Uuid },
    /// An SSH session was closed.
    SshSessionClosed { session_id: Uuid },
    /// Custom event from a module.
    Custom {
        source: String,
        kind: String,
        payload: serde_json::Value,
    },
}
