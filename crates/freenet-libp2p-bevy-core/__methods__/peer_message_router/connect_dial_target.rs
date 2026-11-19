use crate::identity;
use crate::p2p;
use crate::testing;

pub fn connect_dial_target(
    router: &mut testing::PeerMessageRouter,
    from: &identity::PeerId,
    peer: identity::Peer,
) -> Vec<testing::RoutedEvent> {
    if peer.id == *from {
        return vec![testing::RoutedEvent {
            to: from.clone(),
            event: p2p::Libp2pEvent::DialFailed {
                peer_id: peer.id,
                reason: String::from("fake network: cannot dial self"),
            },
        }];
    }
    if !router.registered_peers.contains(&peer.id) {
        return vec![testing::RoutedEvent {
            to: from.clone(),
            event: p2p::Libp2pEvent::DialFailed {
                peer_id: peer.id,
                reason: String::from("fake network: unknown peer"),
            },
        }];
    }
    let already_connected = router
        .connected_peers
        .get(from)
        .is_some_and(|peers| peers.contains(&peer.id));
    if already_connected {
        return Vec::new();
    }
    let target = peer.id;
    router
        .connected_peers
        .entry(from.clone())
        .or_default()
        .insert(target.clone());
    router
        .connected_peers
        .entry(target.clone())
        .or_default()
        .insert(from.clone());
    vec![
        testing::RoutedEvent {
            to: from.clone(),
            event: p2p::Libp2pEvent::Libp2pConnectionEstablished {
                peer_id: target.clone(),
                kind: p2p::ConnectionKind::Tcp,
            },
        },
        testing::RoutedEvent {
            to: from.clone(),
            event: p2p::Libp2pEvent::PeerConnected(target.clone()),
        },
        testing::RoutedEvent {
            to: target.clone(),
            event: p2p::Libp2pEvent::Libp2pConnectionEstablished {
                peer_id: from.clone(),
                kind: p2p::ConnectionKind::Tcp,
            },
        },
        testing::RoutedEvent {
            to: target,
            event: p2p::Libp2pEvent::PeerConnected(from.clone()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::connect_dial_target;
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let dialer = testing::registered_peer(&mut router, "a");
        let target = testing::peer("b", &[]);
        router.register_peer(&target);
        let events = connect_dial_target(&mut router, &dialer, target.clone());
        assert_eq!(
            events,
            vec![
                testing::RoutedEvent {
                    to: dialer.clone(),
                    event: p2p::Libp2pEvent::Libp2pConnectionEstablished {
                        peer_id: target.id.clone(),
                        kind: p2p::ConnectionKind::Tcp,
                    },
                },
                testing::RoutedEvent {
                    to: dialer.clone(),
                    event: p2p::Libp2pEvent::PeerConnected(target.id.clone()),
                },
                testing::RoutedEvent {
                    to: target.id.clone(),
                    event: p2p::Libp2pEvent::Libp2pConnectionEstablished {
                        peer_id: dialer.clone(),
                        kind: p2p::ConnectionKind::Tcp,
                    },
                },
                testing::RoutedEvent {
                    to: target.id,
                    event: p2p::Libp2pEvent::PeerConnected(dialer.clone()),
                },
            ]
        );
    }

    #[test]
    fn dialing_an_unknown_peer_fails_for_the_dialer() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let dialer = testing::registered_peer(&mut router, "a");
        let ghost = testing::peer("ghost", &[]);
        let ghost_id = ghost.id.clone();
        let events = connect_dial_target(&mut router, &dialer, ghost);
        assert!(matches!(
            events.as_slice(),
            [testing::RoutedEvent {
                event: p2p::Libp2pEvent::DialFailed { peer_id, .. },
                ..
            }] if peer_id == &ghost_id
        ));
    }

    #[test]
    fn dialing_self_fails_for_the_dialer() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let dialer = testing::registered_peer(&mut router, "a");
        let events = connect_dial_target(&mut router, &dialer, testing::peer("a", &[]));
        assert!(matches!(
            events.as_slice(),
            [testing::RoutedEvent {
                event: p2p::Libp2pEvent::DialFailed { .. },
                ..
            }]
        ));
    }

    #[test]
    fn redialing_a_connected_peer_sends_nothing() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let dialer = testing::registered_peer(&mut router, "a");
        let target = testing::peer("b", &[]);
        router.register_peer(&target);
        assert_eq!(
            connect_dial_target(&mut router, &dialer, target.clone()).len(),
            4
        );
        assert_eq!(
            connect_dial_target(&mut router, &dialer, target.clone()),
            Vec::new()
        );
        assert_eq!(
            connect_dial_target(&mut router, &target.id, testing::peer("a", &[])),
            Vec::new()
        );
    }
}
