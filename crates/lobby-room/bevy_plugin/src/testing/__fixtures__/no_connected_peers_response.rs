use freenet_stdlib::client_api::{HostResponse, QueryResponse};

#[must_use]
pub const fn no_connected_peers_response() -> HostResponse {
    HostResponse::QueryResponse(QueryResponse::ConnectedPeers { peers: Vec::new() })
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::client_api::{HostResponse, QueryResponse};

    use super::no_connected_peers_response;

    #[test]
    fn test_usage() {
        assert!(matches!(
            no_connected_peers_response(),
            HostResponse::QueryResponse(QueryResponse::ConnectedPeers { peers }) if peers.is_empty()
        ));
    }
}
