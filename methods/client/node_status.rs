use freenet_stdlib::client_api::{
    ClientRequest, HostResponse, NodeDiagnosticsConfig, NodeQuery, QueryResponse,
};

use crate::discovery;
use discovery::freenet::basic::constants;

pub async fn node_status(
    state: &mut discovery::freenet::Client,
) -> Result<discovery::freenet::NodeStatus, discovery::Error> {
    state.send(&ClientRequest::NodeQueries(NodeQuery::NodeDiagnostics {
        config: NodeDiagnosticsConfig::basic_status(),
    }))?;
    let reply = state
        .recv_reply(
            discovery::FreenetRequestKind::QueryNodeStatus,
            constants::REQUEST_TIMEOUT,
        )
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
    use super::node_status;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = discovery::freenet::Client { write, read };
        assert!(node_status(&mut client).await.is_err());
    }
}
