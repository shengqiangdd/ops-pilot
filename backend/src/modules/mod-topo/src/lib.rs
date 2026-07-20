//! mod-topo: Network topology discovery module.
//!
//! Uses SSH to discover network interfaces, routing tables, and service
//! dependencies across managed hosts. Produces a directed graph that can
//! be visualized via ReactFlow on the frontend.

pub mod discovery;
pub mod graph;
pub mod scanner;

use std::sync::Arc;

use async_trait::async_trait;
use ops_pilot_core::ssh::SshConnectionPool;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use tokio::sync::RwLock;
use tracing::info;

pub struct ModTopo {
    graph: Arc<RwLock<graph::TopologyGraph>>,
    executor: Arc<ops_pilot_core::ssh::CommandExecutor>,
}

impl ModTopo {
    pub fn new(ssh_pool: Arc<SshConnectionPool>) -> Self {
        Self {
            graph: Arc::new(RwLock::new(graph::TopologyGraph::new())),
            executor: Arc::new(ops_pilot_core::ssh::CommandExecutor::new(ssh_pool)),
        }
    }
}

#[async_trait]
impl OpsModule for ModTopo {
    fn name(&self) -> &str {
        "mod-topo"
    }

    fn description(&self) -> &str {
        "Network topology discovery and visualization"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "topo_discover".into(),
                description: "Discover network topology for a host or the entire fleet".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": {"type": "string", "description": "Optional host ID to scope discovery"}
                    }
                }),
            },
            ToolDefinition {
                name: "topo_get_graph".into(),
                description: "Get the current topology graph as JSON".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ]
    }

    async fn execute(
        &self,
        _ctx: &ModuleContext,
        tool: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        match tool {
            "topo_discover" => {
                let host_id = params.get("host_id").and_then(|v| v.as_str());
                let discovered = self.discover(host_id).await?;
                Ok(serde_json::json!({ "nodes": discovered }))
            }
            "topo_get_graph" => {
                let g = self.graph.read().await;
                Ok(serde_json::to_value(&*g)?)
            }
            _ => Err(anyhow::anyhow!("unknown tool: {}", tool)),
        }
    }

    async fn on_event(&self, _ctx: &ModuleContext, _event: &OpsEvent) -> Option<ModuleAction> {
        None
    }

    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl ModTopo {
    async fn discover(
        &self,
        host_id: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let mut graph = self.graph.write().await;
        graph.clear();

        let results = discovery::discover_topology(&self.executor, host_id).await?;

        for node in &results.nodes {
            graph.add_node(node.clone());
        }
        for edge in &results.edges {
            graph.add_edge(edge.clone());
        }

        info!(
            nodes = graph.nodes.len(),
            edges = graph.edges.len(),
            "Topology discovery complete"
        );

        let nodes: Vec<serde_json::Value> = graph
            .nodes
            .iter()
            .map(|n| serde_json::to_value(n).unwrap_or_default())
            .collect();
        Ok(nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ops_pilot_core::ssh::SshConnectionPool;

    #[test]
    fn test_module_metadata() {
        let m = ModTopo::new(Arc::new(SshConnectionPool::new()));
        assert_eq!(m.name(), "mod-topo");
        assert!(m.description().contains("topology"));
    }

    #[test]
    fn test_tools_registered() {
        let m = ModTopo::new(Arc::new(SshConnectionPool::new()));
        let tools = m.tools();
        assert_eq!(tools.len(), 2);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"topo_discover"));
        assert!(names.contains(&"topo_get_graph"));
    }
}
