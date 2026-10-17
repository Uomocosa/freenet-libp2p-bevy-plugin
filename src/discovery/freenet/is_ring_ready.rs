use crate::identity;

#[must_use]
pub const fn is_ring_ready(peers: &[identity::FreenetPeer]) -> bool {
    !peers.is_empty()
}

#[cfg(test)]
mod tests {
    use super::is_ring_ready;
    use crate::identity;

    fn peer(port: u16) -> identity::FreenetPeer {
        identity::FreenetPeer {
            id: identity::FreenetPeerId::from("p"),
            addr: std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        }
    }

    #[test]
    fn test_usage() {
        assert!(!is_ring_ready(&[]));
        assert!(is_ring_ready(&[peer(1)]));
    }
}
