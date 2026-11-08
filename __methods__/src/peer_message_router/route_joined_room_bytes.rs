use crate::identity;
use crate::testing;

pub fn route_joined_room_bytes(
    router: &mut testing::PeerMessageRouter,
    from: &identity::PeerId,
    data: Vec<u8>,
) -> Vec<testing::RoutedEvent> {
    let Some(room) = router.joined_rooms.get(from).cloned() else {
        return Vec::new();
    };
    router.route_specific_room_bytes(from, &room, data)
}

#[cfg(test)]
mod tests {
    use super::route_joined_room_bytes;
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

    #[test]
    fn test_usage() {
        let game = identity::GameId::from("game");
        let mut router = testing::PeerMessageRouter::new(game.clone());
        let publisher = registered(&mut router, "a");
        let subscriber = registered(&mut router, "b");
        let room = identity::RoomName::from("room-a");
        for peer in [&publisher, &subscriber] {
            router.route(
                peer,
                p2p::Libp2pCommand::SubscribeRoom { room: room.clone() },
            );
        }
        router.route(
            &publisher,
            p2p::Libp2pCommand::Dial(identity::Peer {
                id: subscriber.clone(),
                addrs: Vec::new(),
            }),
        );
        let events = route_joined_room_bytes(&mut router, &publisher, vec![4, 5]);
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: subscriber,
                event: p2p::Libp2pEvent::Gossip {
                    topic: identity::Topic::from_room(&game, &room),
                    from: publisher,
                    data: vec![4, 5],
                },
            }]
        );
    }

    #[test]
    fn publishing_without_a_joined_room_sends_nothing() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let publisher = registered(&mut router, "a");
        let events = route_joined_room_bytes(&mut router, &publisher, vec![1]);
        assert_eq!(events, Vec::new());
    }
}
