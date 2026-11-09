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

    fn registered(router: &mut testing::PeerMessageRouter, id: &str) -> identity::Peer {
        let peer = identity::Peer {
            id: identity::PeerId::from(id),
            addrs: Vec::new(),
        };
        router.register_peer(&peer);
        peer
    }

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let dialer = registered(&mut router, "a");
        let target = registered(&mut router, "b");
        let events = connect_dial_target(&mut router, &dialer.id, target.clone());
        assert_eq!(
            events,
            vec![
                testing::RoutedEvent {
                    to: dialer.id.clone(),
                    event: p2p::Libp2pEvent::Libp2pConnectionEstablished {
                        peer_id: target.id.clone(),
                        kind: p2p::ConnectionKind::Tcp,
                    },
                },
                testing::RoutedEvent {
                    to: dialer.id.clone(),
                    event: p2p::Libp2pEvent::PeerConnected(target.id.clone()),
                },
                testing::RoutedEvent {
                    to: target.id.clone(),
                    event: p2p::Libp2pEvent::Libp2pConnectionEstablished {
                        peer_id: dialer.id.clone(),
                        kind: p2p::ConnectionKind::Tcp,
                    },
                },
                testing::RoutedEvent {
                    to: target.id,
                    event: p2p::Libp2pEvent::PeerConnected(dialer.id.clone()),
                },
            ]
        );
    }

    #[test]
    fn dialing_an_unknown_peer_fails_for_the_dialer() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let dialer = registered(&mut router, "a");
        let ghost = identity::Peer {
            id: identity::PeerId::from("ghost"),
            addrs: Vec::new(),
        };
        let ghost_id = ghost.id.clone();
        let events = connect_dial_target(&mut router, &dialer.id, ghost);
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
        let dialer = registered(&mut router, "a");
        let events = connect_dial_target(&mut router, &dialer.id, dialer.clone());
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
        let dialer = registered(&mut router, "a");
        let target = registered(&mut router, "b");
        assert_eq!(
            connect_dial_target(&mut router, &dialer.id, target.clone()).len(),
            4
        );
        assert_eq!(
            connect_dial_target(&mut router, &dialer.id, target.clone()),
            Vec::new()
        );
        assert_eq!(
            connect_dial_target(&mut router, &target.id, dialer),
            Vec::new()
        );
    }
}
