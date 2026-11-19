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
    use freenet_stdlib::client_api::{ContractResponse, HostResponse};

    use crate::freenet;
    use crate::testing;

    use super::node_status;

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
        let key = testing::contract_key();
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
            .send(Ok(testing::reachable_diagnostics_response()))
            .ok();
        read_tx
            .send(Ok(testing::no_connected_peers_response()))
            .ok();
        let mut client = freenet::Client { write, read };
        assert!(node_status(&mut client).await.is_ok());
    }

    #[tokio::test]
    async fn late_fetch_reply_does_not_break_node_status() {
        let key = testing::contract_key();
        let state = testing::TestState(9);
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx
            .send(Ok(testing::contract_get_response(key, &state)))
            .ok();
        read_tx
            .send(Ok(testing::contract_not_found_response(key)))
            .ok();
        read_tx
            .send(Ok(testing::reachable_diagnostics_response()))
            .ok();
        read_tx
            .send(Ok(testing::no_connected_peers_response()))
            .ok();
        let mut client = freenet::Client { write, read };
        assert!(node_status(&mut client).await.is_ok());
    }
}
