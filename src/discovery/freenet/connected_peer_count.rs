use freenet_stdlib::client_api::{ClientRequest, HostResponse, NodeQuery, QueryResponse};

use crate::discovery;

pub async fn connected_peer_count(
    client: &mut discovery::freenet::Client,
) -> Result<usize, discovery::Error> {
    client.send(&ClientRequest::NodeQueries(NodeQuery::ConnectedPeers))?;
    match client.recv_response().await? {
        HostResponse::QueryResponse(QueryResponse::ConnectedPeers { peers }) => Ok(peers.len()),
        other => Err(discovery::Error::UnexpectedResponse(format!("{other:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::connected_peer_count;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = discovery::freenet::Client { write, read };
        assert!(connected_peer_count(&mut client).await.is_err());
    }
}
