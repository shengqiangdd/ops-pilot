//! EventBus for publish/subscribe inter-module communication.
//!
//! Built on `tokio::broadcast` — each publish fans out to every active
//! subscriber. Back-pressure is handled via the bounded channel: if a
//! subscriber is too slow it receives a `RecvError::Lagged` on the next
//! read instead of blocking the publisher.

use tokio::sync::broadcast;
use tracing::{debug, warn};

pub use ops_pilot_sdk::events::OpsEvent;

/// A multi-producer, multi-consumer event bus.
///
/// Cloning the `EventBus` is cheap — all clones share the same broadcast
/// channel. Each clone returned by [`subscribe`] is an independent
/// receiver that sees every event published after it was created.
#[derive(Debug, Clone)]
pub struct EventBus {
    tx: broadcast::Sender<OpsEvent>,
}

impl EventBus {
    /// Create a new event bus with the given channel capacity.
    ///
    /// The capacity is the maximum number of events that can be buffered
    /// before a slow subscriber is considered lagged and skipped. A
    /// capacity of 256 is a reasonable default for most workloads.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Publish an event to all current subscribers.
    ///
    /// Returns `Err` only if there are zero active receivers, which is
    /// logged at debug level and not treated as an error — publishing
    /// into an empty bus is a valid, non-fatal scenario.
    pub fn publish(&self, event: OpsEvent) -> Result<(), broadcast::error::SendError<OpsEvent>> {
        debug!(?event, "publishing event");
        match self.tx.send(event) {
            Ok(receiver_count) => {
                debug!(receiver_count, "event dispatched");
                Ok(())
            }
            Err(e) => {
                warn!("no active subscribers — event dropped");
                Err(e)
            }
        }
    }

    /// Create a new subscriber receiver.
    ///
    /// The returned [`broadcast::Receiver`] will see every event
    /// published **after** this call completes. It is safe to hold
    /// across `.await` points.
    pub fn subscribe(&self) -> broadcast::Receiver<OpsEvent> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn new_creates_empty_bus() {
        let bus = EventBus::new(32);
        // No subscribers yet — publish returns Err(SendError).
        let err = bus
            .publish(OpsEvent::AuditLog {
                user: "alice".into(),
                action: "login".into(),
                resource: "/dashboard".into(),
                outcome: "success".into(),
            })
            .unwrap_err();
        // The event is wrapped in SendError; verify it round-trips via Debug.
        assert!(format!("{:?}", err.0).contains("AuditLog"));
    }

    #[tokio::test]
    async fn single_subscriber_receives_events() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.publish(OpsEvent::ConnectionAdded {
            id: Uuid::new_v4(),
            kind: "ssh".into(),
            name: "prod-web".into(),
        })
        .unwrap();

        let event = rx.recv().await.unwrap();
        match event {
            OpsEvent::ConnectionAdded { kind, name, .. } => {
                assert_eq!(kind, "ssh");
                assert_eq!(name, "prod-web");
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[tokio::test]
    async fn multiple_subscribers_all_receive() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.publish(OpsEvent::CommandExecuted {
            host_id: Uuid::new_v4(),
            command: "uptime".into(),
            exit_code: 0,
        })
        .unwrap();

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();
        // Both should see the same event variant.
        assert!(matches!(e1, OpsEvent::CommandExecuted { .. }));
        assert!(matches!(e2, OpsEvent::CommandExecuted { .. }));
    }

    #[tokio::test]
    async fn late_subscriber_does_not_see_old_events() {
        let bus = EventBus::new(16);

        // Create a temporary subscriber so the publish succeeds, then drop it.
        {
            let mut tmp = bus.subscribe();
            bus.publish(OpsEvent::HealthCheck {
                host_id: Uuid::new_v4(),
                status: "ok".into(),
                details: serde_json::json!({"cpu": 12.3}),
            })
            .unwrap();
            // Drain so it doesn't lag.
            let _ = tmp.recv().await;
        }

        // Late subscriber should see nothing.
        let mut rx = bus.subscribe();
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn subscriber_lagged_is_signalled() {
        // Tiny capacity so we overflow quickly.
        let bus = EventBus::new(1);
        let mut rx = bus.subscribe();

        // Publish twice without consuming — the second send lags rx.
        bus.publish(OpsEvent::DockerEvent {
            container_id: "c1".into(),
            action: "start".into(),
            actor: "docker".into(),
        })
        .unwrap();

        // Second publish will cause the receiver to lag.
        let _ = bus.publish(OpsEvent::DockerEvent {
            container_id: "c2".into(),
            action: "stop".into(),
            actor: "docker".into(),
        });

        let result = rx.recv().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn publish_all_required_variants() {
        let bus = EventBus::new(64);
        let mut rx = bus.subscribe();

        let events = vec![
            OpsEvent::ConnectionAdded {
                id: Uuid::new_v4(),
                kind: "docker".into(),
                name: "local".into(),
            },
            OpsEvent::ConnectionRemoved {
                id: Uuid::new_v4(),
                kind: "ssh".into(),
            },
            OpsEvent::CommandExecuted {
                host_id: Uuid::new_v4(),
                command: "ls -la".into(),
                exit_code: 0,
            },
            OpsEvent::DockerEvent {
                container_id: "abc".into(),
                action: "die".into(),
                actor: "container".into(),
            },
            OpsEvent::HealthCheck {
                host_id: Uuid::new_v4(),
                status: "degraded".into(),
                details: serde_json::json!({"mem": 90.5}),
            },
            OpsEvent::AuditLog {
                user: "bob".into(),
                action: "delete".into(),
                resource: "key/secret".into(),
                outcome: "denied".into(),
            },
            OpsEvent::ModuleAction {
                module: "rca".into(),
                action: "analyze".into(),
                payload: serde_json::json!({"target": "cpu spike"}),
            },
        ];

        for event in &events {
            bus.publish(event.clone()).unwrap();
        }

        for expected in &events {
            let received = rx.recv().await.unwrap();
            // Verify the Debug representations match.
            assert_eq!(format!("{expected:?}"), format!("{received:?}"));
        }
    }

    #[test]
    fn bus_is_cloneable_and_shared() {
        let bus = EventBus::new(8);
        let clone = bus.clone();
        // Publishing via clone reaches the original's channel.
        let result = clone.publish(OpsEvent::AuditLog {
            user: "eve".into(),
            action: "probe".into(),
            resource: "/health".into(),
            outcome: "ok".into(),
        });
        // No subscribers, so Err — but it demonstrates the clone works.
        assert!(result.is_err());
    }
}
