//! Topology discovery engine.
//!
//! Uses SSH to probe network interfaces, ARP tables, routing tables,
//! and listening services on remote hosts.

use std::collections::HashMap;

use ops_pilot_core::ssh::CommandExecutor;
use tracing::{info, warn};

use crate::graph::{NodeKind, TopoEdge, TopoNode};

/// Results of a topology discovery scan.
pub struct DiscoveryResult {
    pub nodes: Vec<TopoNode>,
    pub edges: Vec<TopoEdge>,
}

/// Discover network topology for a host or all hosts.
pub async fn discover_topology(
    executor: &CommandExecutor,
    host_id: Option<&str>,
) -> anyhow::Result<DiscoveryResult> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    match host_id {
        Some(hid) => {
            match executor.exec_on_host(hid, "ip addr show").await {
                Ok(result) if result.success() => {
                    let host_node = parse_host_from_ip(hid, &result.stdout);
                    nodes.push(host_node);
                    discover_services(executor, hid, &mut nodes, &mut edges).await;
                    discover_neighbors(executor, hid, &mut nodes, &mut edges).await;
                }
                Ok(result) => {
                    warn!(host_id = hid, stderr = %result.stderr, "ip addr command failed");
                }
                Err(e) => {
                    warn!(host_id = hid, error = %e, "SSH exec failed for topology discovery");
                }
            }
        }
        None => {
            warn!("topo_discover without host_id requires a host registry; returning empty graph");
        }
    }

    Ok(DiscoveryResult { nodes, edges })
}

/// Parse `ip addr show` output into a TopoNode.
fn parse_host_from_ip(host_id: &str, output: &str) -> TopoNode {
    let mut properties = HashMap::new();
    let mut ips = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("inet ") {
            if let Some(ip) = trimmed.split_whitespace().nth(1) {
                if let Some(cidr) = ip.split('/').next() {
                    ips.push(cidr.to_string());
                    properties.insert(format!("ip_{}", ips.len()), cidr.to_string());
                }
            }
        }
    }

    TopoNode {
        id: host_id.to_string(),
        label: host_id.to_string(),
        kind: NodeKind::Host,
        properties,
    }
}

/// Discover listening services via `ss -tlnp`.
async fn discover_services(
    executor: &CommandExecutor,
    host_id: &str,
    nodes: &mut Vec<TopoNode>,
    edges: &mut Vec<TopoEdge>,
) {
    let cmd = "ss -tlnp 2>/dev/null || netstat -tlnp 2>/dev/null";
    match executor.exec_on_host(host_id, cmd).await {
        Ok(result) if result.success() => {
            for line in result.stdout.lines().skip(1) {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 4 {
                    let local_addr = parts[3];
                    let (port, service_name) = if let Some(colon_pos) = local_addr.rfind(':') {
                        let port_str = &local_addr[colon_pos + 1..];
                        let svc = match port_str.parse::<u16>() {
                            Ok(22) => "ssh",
                            Ok(80) => "http",
                            Ok(443) => "https",
                            Ok(3306) => "mysql",
                            Ok(5432) => "postgresql",
                            Ok(6379) => "redis",
                            Ok(8080) => "http-alt",
                            Ok(8443) => "https-alt",
                            Ok(27017) => "mongodb",
                            Ok(p) => {
                                if p >= 3000 && p <= 9999 {
                                    "app"
                                } else {
                                    "service"
                                }
                            }
                            Err(_) => "unknown",
                        };
                        (port_str.parse::<u16>().ok(), svc)
                    } else {
                        (None, "service")
                    };

                    let svc_id = format!("{}:{}", host_id, service_name);
                    nodes.push(TopoNode {
                        id: svc_id.clone(),
                        label: service_name.to_string(),
                        kind: NodeKind::Service,
                        properties: {
                            let mut p = HashMap::new();
                            if let Some(port) = port {
                                p.insert("port".to_string(), port.to_string());
                            }
                            p.insert("host".to_string(), host_id.to_string());
                            p
                        },
                    });
                    edges.push(TopoEdge {
                        source: host_id.to_string(),
                        target: svc_id,
                        label: format!("runs {}", service_name),
                        protocol: Some("tcp".to_string()),
                        port,
                    });
                }
            }
        }
        Ok(_) | Err(_) => {
            info!(host_id, "Could not discover services (ss/netstat unavailable)");
        }
    }
}

/// Discover network neighbors via ARP table.
async fn discover_neighbors(
    executor: &CommandExecutor,
    host_id: &str,
    nodes: &mut Vec<TopoNode>,
    edges: &mut Vec<TopoEdge>,
) {
    let cmd = "arp -a 2>/dev/null || ip neigh show 2>/dev/null";
    match executor.exec_on_host(host_id, cmd).await {
        Ok(result) if result.success() => {
            for line in result.stdout.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                // Parse arp output: hostname (ip) at mac on iface
                if let Some(paren_start) = trimmed.find('(') {
                    if let Some(paren_end) = trimmed.find(')') {
                        let ip = &trimmed[paren_start + 1..paren_end];
                        if ip != "incomplete" && !ip.is_empty() {
                            let neighbor_id = format!("host-{}", ip.replace('.', "_"));
                            if !nodes.iter().any(|n| n.id == neighbor_id) {
                                nodes.push(TopoNode {
                                    id: neighbor_id.clone(),
                                    label: ip.to_string(),
                                    kind: NodeKind::Host,
                                    properties: {
                                        let mut p = HashMap::new();
                                        p.insert("ip".to_string(), ip.to_string());
                                        p
                                    },
                                });
                            }
                            let edge_label = extract_iface(trimmed);
                            edges.push(TopoEdge {
                                source: host_id.to_string(),
                                target: neighbor_id,
                                label: edge_label,
                                protocol: None,
                                port: None,
                            });
                        }
                    }
                }
            }
        }
        Ok(_) | Err(_) => {
            info!(host_id, "Could not discover neighbors (arp unavailable)");
        }
    }
}

fn extract_iface(line: &str) -> String {
    if let Some(on_pos) = line.find(" on ") {
        let rest = &line[on_pos + 4..];
        if let Some(iface) = rest.split_whitespace().next() {
            return format!("via {}", iface);
        }
    }
    "layer-2".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_host_from_ip() {
        let output = r#"1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN
    inet 127.0.0.1/8 scope host lo
    inet6 ::1/128 scope host
2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc mq state UP
    inet 10.0.0.15/24 brd 10.0.0.255 scope global eth0
    inet6 fe80::1/64 scope link eth0"#;

        let node = parse_host_from_ip("web-server-1", output);
        assert_eq!(node.id, "web-server-1");
        assert_eq!(node.kind, NodeKind::Host);
        assert!(node.properties.contains_key("ip_1"));
        assert!(node.properties.contains_key("ip_2"));
    }
}
