use freenet_stdlib::client_api::{
    ClientRequest, HostResponse, NodeDiagnosticsConfig, NodeQuery, QueryResponse,
};

use crate::discovery;

pub async fn node_status(
    state: &mut discovery::freenet::Client,
) -> Result<discovery::freenet::NodeStatus, discovery::Error> {
    state.send(&ClientRequest::NodeQueries(NodeQuery::NodeDiagnostics {
        config: NodeDiagnosticsConfig::basic_status(),
    }))?;
    let reply = state
        .recv_reply(discovery::freenet::RequestKind::QueryNodeStatus)
        .await?;
    let HostResponse::QueryResponse(QueryResponse::NodeDiagnostics(diagnostics)) = reply else {
        return Err(discovery::Error::UnexpectedResponse(format!("{reply:?}")));
    };
    let peers = state.connected_peers().await?;
    Ok(discovery::freenet::NodeStatus::from_diagnostics(
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

    use super::node_status;
    use crate::discovery;

    fn contract_key() -> ContractKey {
        WrappedContract::new(
            Arc::new(ContractCode::from(
                discovery::freenet::LOBBY_CONTRACT_WASM.to_vec(),
            )),
            Parameters::from(Vec::new()),
        )
        .key
    }

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = discovery::freenet::Client { write, read };
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
        let mut client = discovery::freenet::Client { write, read };
        assert!(node_status(&mut client).await.is_ok());
    }
}
