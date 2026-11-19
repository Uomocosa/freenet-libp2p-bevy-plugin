use std::collections::HashMap;

use freenet_stdlib::client_api::{NetworkInfo, NodeDiagnosticsResponse, NodeInfo};

#[must_use]
pub fn node_diagnostics(
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
        contract_states: HashMap::new(),
        system_metrics: None,
        connected_peers_detailed: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::node_diagnostics;

    #[test]
    fn test_usage() {
        let diagnostics = node_diagnostics(Some("0.5"), 27, true);
        assert_eq!(
            diagnostics
                .network_info
                .as_ref()
                .map(|network| network.active_connections),
            Some(27)
        );
        assert!(diagnostics.node_info.is_some_and(|info| info.is_gateway));
    }
}
