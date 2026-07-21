//! Topology graph data structures.
//!
//! Represents the network as a directed graph of hosts, services,
//! and the connections between them.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A node in the topology graph (host, service, network device).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopoNode {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
    pub properties: HashMap<String, String>,
}

/// An edge between two topology nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopoEdge {
    pub source: String,
    pub target: String,
    pub label: String,
    pub protocol: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeKind {
    Host,
    Service,
    Container,
    LoadBalancer,
    Database,
    External,
}

/// The full topology graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyGraph {
    pub nodes: Vec<TopoNode>,
    pub edges: Vec<TopoEdge>,
}

impl Default for TopologyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl TopologyGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: TopoNode) {
        if !self.nodes.iter().any(|n| n.id == node.id) {
            self.nodes.push(node);
        }
    }

    pub fn add_edge(&mut self, edge: TopoEdge) {
        self.edges.push(edge);
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }
}
