//! Port scanner — lightweight TCP connect scan to discover open ports on a host.

use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tracing::info;

use crate::graph::{NodeKind, TopoEdge, TopoNode};

/// Common well-known ports to scan.
const COMMON_PORTS: &[(u16, &str)] = &[
    (22, "ssh"),
    (80, "http"),
    (443, "https"),
    (3306, "mysql"),
    (5432, "postgresql"),
    (6379, "redis"),
    (8080, "http-alt"),
    (8443, "https-alt"),
    (27017, "mongodb"),
    (9090, "prometheus"),
    (2379, "etcd"),
    (6443, "kubernetes"),
    (5601, "kibana"),
    (9200, "elasticsearch"),
];

/// Scan a host for open ports using TCP connect.
pub async fn scan_host(host: &str, ports: Option<&[u16]>) -> Vec<(u16, String)> {
    let default_ports: Vec<u16> = COMMON_PORTS.iter().map(|(p, _)| *p).collect();
    let ports_to_scan = ports.unwrap_or(&default_ports);

    let mut open_ports = Vec::new();

    for &port in ports_to_scan {
        let addr = format!("{}:{}", host, port);
        if let Ok(Ok(_)) = timeout(Duration::from_millis(500), TcpStream::connect(&addr)).await {
            let service = COMMON_PORTS
                .iter()
                .find(|(p, _)| *p == port)
                .map(|(_, s)| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            info!(host, port, service = %service, "port open");
            open_ports.push((port, service));
        }
    }

    open_ports
}

/// Create topology nodes and edges from scan results.
pub fn scan_results_to_graph(
    host_id: &str,
    open_ports: &[(u16, String)],
) -> (Vec<TopoNode>, Vec<TopoEdge>) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for (port, service) in open_ports {
        let svc_id = format!("{}:{}", host_id, service);
        nodes.push(TopoNode {
            id: svc_id.clone(),
            label: service.clone(),
            kind: NodeKind::Service,
            properties: {
                let mut p = std::collections::HashMap::new();
                p.insert("port".to_string(), port.to_string());
                p.insert("host".to_string(), host_id.to_string());
                p
            },
        });
        edges.push(TopoEdge {
            source: host_id.to_string(),
            target: svc_id,
            label: format!("{}:{}", host_id, port),
            protocol: Some("tcp".to_string()),
            port: Some(*port),
        });
    }

    (nodes, edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_results_to_graph() {
        let ports = vec![(22, "ssh".to_string()), (80, "http".to_string())];
        let (nodes, edges) = scan_results_to_graph("server-1", &ports);
        assert_eq!(nodes.len(), 2);
        assert_eq!(edges.len(), 2);
        assert_eq!(edges[0].source, "server-1");
    }
}
