use freenet_stdlib::client_api::{ClientRequest, HostResponse, NodeQuery, QueryResponse};

use crate::freenet;
use crate::identity;

pub async fn connected_peers(
    state: &mut freenet::Client,
) -> Result<Vec<identity::FreenetPeer>, freenet::Error> {
    state.send(&ClientRequest::NodeQueries(NodeQuery::ConnectedPeers))?;
    let reply = state.recv_reply(freenet::RequestKind::QueryPeers).await?;
    match reply {
        HostResponse::QueryResponse(QueryResponse::ConnectedPeers { peers }) => Ok(peers
            .into_iter()
            .map(|(peer, addr)| identity::FreenetPeer {
                id: identity::FreenetPeerId::from(peer.as_str()),
                addr,
            })
            .collect()),
        other => Err(freenet::Error::UnexpectedResponse(format!("{other:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::connected_peers;
    use crate::freenet;

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = freenet::Client { write, read };
        assert!(connected_peers(&mut client).await.is_err());
    }
}
