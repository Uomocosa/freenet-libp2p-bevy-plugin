use freenet_stdlib::client_api::NodeDiagnosticsResponse;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeStatus {
    pub ring_peers: usize,
    pub transport_peers: usize,
    pub ring_location: Option<f64>,
    pub is_gateway: bool,
}

impl NodeStatus {
    #[must_use]
    pub fn from_diagnostics(diagnostics: &NodeDiagnosticsResponse, transport_peers: usize) -> Self {
        let node_info = diagnostics.node_info.as_ref();
        Self {
            ring_peers: diagnostics
                .network_info
                .as_ref()
                .map_or(0, |network| network.active_connections),
            transport_peers,
            ring_location: node_info
                .and_then(|node| node.location.as_deref())
                .and_then(|raw| raw.parse().ok()),
            is_gateway: node_info.is_some_and(|node| node.is_gateway),
        }
    }
}

#[rustfmt::skip]
impl NodeStatus {
    #[must_use]
    pub const fn is_ring_ready(&self) -> bool { self.ring_peers > 0 }
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::client_api::{NetworkInfo, NodeDiagnosticsResponse, NodeInfo};

    use super::NodeStatus;

    fn diagnostics(
        location: Option<&str>,
        ring_peers: usize,
        is_gateway: bool,
    ) -> NodeDiagnosticsResponse {
        NodeDiagnosticsResponse {
            node_info: Some(NodeInfo {
                peer_id: "peer".to_string(),
                is_gateway,
                location: location.map(str::to_string),
                listening_address: None,
                uptime_seconds: 1,
            }),
            network_info: Some(NetworkInfo {
                connected_peers: Vec::new(),
                active_connections: ring_peers,
            }),
            subscriptions: Vec::new(),
            contract_states: std::collections::HashMap::new(),
            system_metrics: None,
            connected_peers_detailed: Vec::new(),
        }
    }

    #[test]
    fn test_usage() {
        let gateway_only =
            NodeStatus::from_diagnostics(&diagnostics(Some("0.999024"), 0, false), 2);
        assert_eq!(gateway_only.ring_peers, 0);
        assert_eq!(gateway_only.transport_peers, 2);
        assert_eq!(gateway_only.ring_location, Some(0.999_024));
        assert!(!gateway_only.is_ring_ready());

        let joined = NodeStatus::from_diagnostics(&diagnostics(Some("0.5"), 27, true), 29);
        assert!(joined.is_ring_ready());
        assert!(joined.is_gateway);

        let unknown = NodeStatus::from_diagnostics(
            &NodeDiagnosticsResponse {
                node_info: None,
                network_info: None,
                subscriptions: Vec::new(),
                contract_states: std::collections::HashMap::new(),
                system_metrics: None,
                connected_peers_detailed: Vec::new(),
            },
            0,
        );
        assert_eq!(unknown, NodeStatus::default());
    }
}
