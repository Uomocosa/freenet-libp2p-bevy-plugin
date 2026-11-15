use freenet_stdlib::client_api::{
    ClientRequest, HostResponse, NodeDiagnosticsConfig, NodeQuery, QueryResponse,
};

use crate::freenet;

pub async fn node_status(
    state: &mut freenet::Client,
) -> Result<freenet::NodeStatus, freenet::Error> {
    state.send(&ClientRequest::NodeQueries(NodeQuery::NodeDiagnostics {
        config: NodeDiagnosticsConfig::basic_status(),
    }))?;
    let reply = state
        .recv_reply(freenet::RequestKind::QueryNodeStatus)
        .await?;
    let HostResponse::QueryResponse(QueryResponse::NodeDiagnostics(diagnostics)) = reply else {
        return Err(freenet::Error::UnexpectedResponse(format!("{reply:?}")));
    };
    let peers = state.connected_peers().await?;
    Ok(freenet::NodeStatus::from_diagnostics(
        &diagnostics,
        peers.len(),
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use freenet_stdlib::client_api::{
        ContractResponse, HostResponse, NetworkInfo, NodeDiagnosticsResponse, NodeInfo,
        QueryResponse,
    };
    use freenet_stdlib::prelude::{ContractCode, ContractKey, Parameters, WrappedContract};
    use serde::{Deserialize, Serialize};

    use super::node_status;
    use crate::freenet;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
    struct TestState {
        value: u64,
    }

    fn contract_key() -> ContractKey {
        WrappedContract::new(
            Arc::new(ContractCode::from(vec![0; 4])),
            Parameters::from(Vec::new()),
        )
        .key
    }

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = freenet::Client { write, read };
        assert!(node_status(&mut client).await.is_err());
    }

    #[tokio::test]
    async fn late_subscribe_response_does_not_break_node_status() {
        let key = contract_key();
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::SubscribeResponse {
                    key,
                    subscribed: true,
                },
            )))
            .ok();
        read_tx
            .send(Ok(HostResponse::QueryResponse(
                QueryResponse::NodeDiagnostics(NodeDiagnosticsResponse {
                    node_info: Some(NodeInfo {
                        peer_id: "peer".to_string(),
                        is_gateway: false,
                        location: Some("0.5".to_string()),
                        listening_address: None,
                        uptime_seconds: 1,
                    }),
                    network_info: Some(NetworkInfo {
                        connected_peers: Vec::new(),
                        active_connections: 1,
                    }),
                    subscriptions: Vec::new(),
                    contract_states: HashMap::new(),
                    system_metrics: None,
                    connected_peers_detailed: Vec::new(),
                }),
            )))
            .ok();
        read_tx
            .send(Ok(HostResponse::QueryResponse(
                QueryResponse::ConnectedPeers { peers: Vec::new() },
            )))
            .ok();
        let mut client = freenet::Client { write, read };
        assert!(node_status(&mut client).await.is_ok());
    }

    #[tokio::test]
    async fn late_fetch_reply_does_not_break_node_status() {
        use freenet_stdlib::prelude::WrappedState;
        let key = contract_key();
        let state = TestState { value: 9 };
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::GetResponse {
                    key,
                    contract: None,
                    state: WrappedState::new(bincode::serialize(&state).unwrap_or_default()),
                },
            )))
            .ok();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::NotFound {
                    instance_id: *key.id(),
                },
            )))
            .ok();
        read_tx
            .send(Ok(HostResponse::QueryResponse(
                QueryResponse::NodeDiagnostics(NodeDiagnosticsResponse {
                    node_info: Some(NodeInfo {
                        peer_id: "peer".to_string(),
                        is_gateway: false,
                        location: Some("0.5".to_string()),
                        listening_address: None,
                        uptime_seconds: 1,
                    }),
                    network_info: Some(NetworkInfo {
                        connected_peers: Vec::new(),
                        active_connections: 1,
                    }),
                    subscriptions: Vec::new(),
                    contract_states: HashMap::new(),
                    system_metrics: None,
                    connected_peers_detailed: Vec::new(),
                }),
            )))
            .ok();
        read_tx
            .send(Ok(HostResponse::QueryResponse(
                QueryResponse::ConnectedPeers { peers: Vec::new() },
            )))
            .ok();
        let mut client = freenet::Client { write, read };
        assert!(node_status(&mut client).await.is_ok());
    }
}
