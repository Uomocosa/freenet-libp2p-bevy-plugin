use crate::identity;
use crate::p2p;
use crate::testing;

pub fn remove_peer(
    router: &mut testing::PeerMessageRouter,
    peer: &identity::PeerId,
) -> Vec<testing::RoutedEvent> {
    if !router.registered_peers.remove(peer) {
        return Vec::new();
    }
    let mut events = Vec::new();
    let disconnected = router.connected_peers.remove(peer).unwrap_or_default();
    for other in disconnected {
        if let Some(peers) = router.connected_peers.get_mut(&other) {
            peers.remove(peer);
        }
        events.push(testing::RoutedEvent {
            to: other,
            event: p2p::Libp2pEvent::PeerDisconnected(peer.clone()),
        });
    }
    router.subscribed_topics.remove(peer);
    router.joined_zones.remove(peer);
    router.watched_zones.remove(peer);
    router
        .pending_snapshot_replies
        .retain(|_, requester| requester != peer);
    events
}
#[cfg(test)]
mod tests {
    use super::remove_peer;
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let dialer = testing::registered_peer(&mut router, "a");
        let target = testing::registered_peer(&mut router, "b");
        let lonely = testing::registered_peer(&mut router, "c");
        router.route(
            &dialer,
            p2p::Libp2pCommand::Dial(identity::Peer {
                id: target.clone(),
                addrs: Vec::new(),
            }),
        );
        let events = remove_peer(&mut router, &target);
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: dialer.clone(),
                event: p2p::Libp2pEvent::PeerDisconnected(target.clone()),
            }]
        );
        let events = router.route(
            &dialer,
            p2p::Libp2pCommand::Dial(identity::Peer {
                id: target,
                addrs: Vec::new(),
            }),
        );
        assert!(matches!(
            events.as_slice(),
            [testing::RoutedEvent {
                event: p2p::Libp2pEvent::DialFailed { .. },
                ..
            }]
        ));
        let events = remove_peer(&mut router, &lonely);
        assert_eq!(events, Vec::new());
    }

    #[test]
    fn removing_an_unknown_peer_returns_no_events() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let events = remove_peer(&mut router, &identity::PeerId::from("ghost"));
        assert_eq!(events, Vec::new());
    }

    #[test]
    fn removal_drops_subscriptions_and_connections() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let dialer = testing::registered_peer(&mut router, "a");
        let target = testing::registered_peer(&mut router, "b");
        router.route(
            &target,
            p2p::Libp2pCommand::Subscribe {
                topic: identity::Topic::from("some/topic"),
            },
        );
        router.route(
            &dialer,
            p2p::Libp2pCommand::Dial(identity::Peer {
                id: target.clone(),
                addrs: Vec::new(),
            }),
        );
        remove_peer(&mut router, &target);
        let events = router.route(
            &target,
            p2p::Libp2pCommand::Publish {
                topic: identity::Topic::from("some/topic"),
                data: vec![1],
            },
        );
        assert_eq!(events, Vec::new());
    }
}
