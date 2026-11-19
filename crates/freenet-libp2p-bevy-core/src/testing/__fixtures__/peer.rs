use crate::identity;

#[must_use]
pub fn peer(id: &str, addrs: &[&str]) -> identity::Peer {
    identity::Peer {
        id: identity::PeerId::from(id),
        addrs: addrs
            .iter()
            .copied()
            .map(identity::PeerAddr::from)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::peer;

    #[test]
    fn test_usage() {
        let peer = peer("a", &["/ip4/127.0.0.1/tcp/5001"]);
        assert_eq!(peer.id.as_str(), "a");
        assert_eq!(peer.addrs.len(), 1);
    }
}
