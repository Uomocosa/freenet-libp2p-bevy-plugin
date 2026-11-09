use crate::identity;
use crate::testing;

pub fn register_peer(router: &mut testing::PeerMessageRouter, peer: &identity::Peer) {
    router.registered_peers.insert(peer.id.clone());
    router.connected_peers.entry(peer.id.clone()).or_default();
    router.subscribed_topics.entry(peer.id.clone()).or_default();
}

#[cfg(test)]
mod tests {
    use super::register_peer;
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = identity::Peer {
            id: identity::PeerId::from("a"),
            addrs: Vec::new(),
        };
        register_peer(&mut router, &peer);
        register_peer(&mut router, &peer);
        let target = identity::Peer {
            id: identity::PeerId::from("b"),
            addrs: Vec::new(),
        };
        router.register_peer(&target);
        let events = router.route(&peer.id, p2p::Libp2pCommand::AddKadPeer(target));
        assert_eq!(events, Vec::new());
    }
}
