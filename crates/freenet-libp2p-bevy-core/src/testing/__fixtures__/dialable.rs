use crate::identity;

#[must_use]
pub fn dialable(peer: &identity::Peer) -> identity::Peer {
    identity::Peer {
        id: peer.id.clone(),
        addrs: peer
            .addrs
            .iter()
            .map(|addr| identity::PeerAddr(addr.replace("/ip4/0.0.0.0/", "/ip4/127.0.0.1/")))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::testing;

    use super::dialable;

    #[test]
    fn test_usage() {
        let peer = testing::peer("a", &["/ip4/0.0.0.0/tcp/5001"]);
        let dialed = dialable(&peer);
        assert_eq!(dialed.id, peer.id);
        assert_eq!(
            dialed.addrs,
            vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001")]
        );
    }
}
