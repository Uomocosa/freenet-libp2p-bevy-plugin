use crate::discovery;
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
    router.joined_rooms.remove(peer);
    router.heartbeat_targets.remove(peer);
    router
        .pending_snapshot_replies
        .retain(|_, requester| requester != peer);
    for room in router.shared_lobby.values_mut() {
        let is_departed = room
            .members
            .get(peer)
            .is_some_and(|member| member.membership == discovery::RoomMembership::Departed);
        if is_departed {
            room.members.remove(peer);
        }
    }
    router.shared_lobby.drop_rooms_where_every_member_quit();
    events
}

#[cfg(test)]
mod tests {
    use super::remove_peer;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    fn registered(router: &mut testing::PeerMessageRouter, id: &str) -> identity::PeerId {
        let peer = identity::Peer {
            id: identity::PeerId::from(id),
            addrs: Vec::new(),
        };
        router.register_peer(&peer);
        peer.id
    }

    fn heartbeat(
        room: &str,
        membership: discovery::RoomMembership,
    ) -> discovery::state_machine::HeartbeatTarget {
        discovery::state_machine::HeartbeatTarget {
            room: identity::RoomName::from(room),
            created_at: discovery::UnixTime::from(10),
            addrs: Vec::new(),
            display_name: identity::Username::from("someone"),
            membership,
        }
    }

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let dialer = registered(&mut router, "a");
        let target = registered(&mut router, "b");
        let lonely = registered(&mut router, "c");
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
        let dialer = registered(&mut router, "a");
        let target = registered(&mut router, "b");
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

    #[test]
    fn crashed_occupant_stays_listed_like_a_lease() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let holder = registered(&mut router, "holder");
        router.refresh_lobby(
            &holder,
            &[heartbeat("room-a", discovery::RoomMembership::Occupant)],
            discovery::UnixTime::from(20),
        );
        remove_peer(&mut router, &holder);
        assert!(
            router
                .shared_lobby
                .get(&identity::RoomName::from("room-a"))
                .is_some_and(|record| record.members.contains_key(&holder)),
            "crashed holder stays listed so a joiner still sees the room"
        );
    }

    #[test]
    fn clean_quit_departed_is_removed() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let quitter = registered(&mut router, "quitter");
        router.refresh_lobby(
            &quitter,
            &[heartbeat("room-a", discovery::RoomMembership::Departed)],
            discovery::UnixTime::from(20),
        );
        remove_peer(&mut router, &quitter);
        assert!(
            router
                .shared_lobby
                .get(&identity::RoomName::from("room-a"))
                .is_none_or(|record| !record.members.contains_key(&quitter)),
            "clean quit removes the departed entry so the room can go"
        );
    }
}
