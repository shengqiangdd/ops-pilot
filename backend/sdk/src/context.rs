use std::path::{Path, PathBuf};
use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::broadcast;

use crate::events::OpsEvent;

/// A lightweight event bus sender backed by `tokio::broadcast`.
///
/// Cloning is cheap — all clones share the same channel.
#[derive(Debug, Clone)]
pub struct EventBus {
    tx: broadcast::Sender<OpsEvent>,
}

impl EventBus {
    /// Create a new event bus with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Publish an event to all subscribers.
    ///
    /// Returns `Err` only if there are zero active receivers.
    pub fn publish(&self, event: OpsEvent) -> Result<(), broadcast::error::SendError<OpsEvent>> {
        self.tx.send(event).map(|_| ())
    }

    /// Create a new subscriber receiver.
    pub fn subscribe(&self) -> broadcast::Receiver<OpsEvent> {
        self.tx.subscribe()
    }
}

/// Shared context passed to module `execute` / `on_event` / `health_check` calls.
///
/// Provides controlled access to core services without coupling modules
/// to specific implementations.
#[derive(Clone)]
pub struct ModuleContext {
    /// Database connection pool.
    db: Arc<SqlitePool>,
    /// Event bus for publishing and subscribing to events.
    event_bus: EventBus,
    /// Directory where this module stores its config files.
    config_path: PathBuf,
    /// Name of the module this context was created for.
    module_name: String,
}

impl ModuleContext {
    /// Create a new `ModuleContext`.
    pub fn new(
        db: Arc<SqlitePool>,
        event_bus: EventBus,
        config_path: PathBuf,
        module_name: String,
    ) -> Self {
        Self {
            db,
            event_bus,
            config_path,
            module_name,
        }
    }

    /// Access the database connection pool.
    pub fn db(&self) -> &SqlitePool {
        &self.db
    }

    /// Publish an event to the event bus.
    pub async fn emit(&self, event: OpsEvent) {
        // Ignore the error — it only fires when there are zero subscribers.
        let _ = self.event_bus.publish(event);
    }

    /// Return the module's configuration directory.
    pub fn config_dir(&self) -> &Path {
        &self.config_path
    }

    /// Return the name of the module this context belongs to.
    pub fn module_name(&self) -> &str {
        &self.module_name
    }
}
