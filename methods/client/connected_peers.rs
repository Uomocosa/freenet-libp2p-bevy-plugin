use freenet_stdlib::client_api::{ClientRequest, HostResponse, NodeQuery, QueryResponse};

use crate::discovery;
use crate::identity;
use discovery::freenet::basic::constants;

pub async fn connected_peers(
    state: &mut discovery::freenet::Client,
) -> Result<Vec<identity::FreenetPeer>, discovery::Error> {
    state.send(&ClientRequest::NodeQueries(NodeQuery::ConnectedPeers))?;
    let reply = discovery::freenet::RequestTimeout(constants::REQUEST_TIMEOUT)
        .within(
            discovery::FreenetRequestKind::QueryPeers,
            state.recv_response(),
        )
        .await?;
    match reply {
        HostResponse::QueryResponse(QueryResponse::ConnectedPeers { peers }) => Ok(peers
            .into_iter()
            .map(|(peer, addr)| identity::FreenetPeer {
                id: identity::FreenetPeerId::from(peer.as_str()),
                addr,
            })
            .collect()),
        other => Err(discovery::Error::UnexpectedResponse(format!("{other:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::connected_peers;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = discovery::freenet::Client { write, read };
        assert!(connected_peers(&mut client).await.is_err());
    }
}
