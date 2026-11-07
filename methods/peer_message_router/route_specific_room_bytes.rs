use crate::identity;
use crate::p2p;
use crate::testing;

pub fn route_specific_room_bytes(
    router: &mut testing::PeerMessageRouter,
    from: &identity::PeerId,
    room: &identity::RoomName,
    data: Vec<u8>,
) -> Vec<testing::RoutedEvent> {
    let topic = identity::Topic::from_room(&router.game, room);
    router.route(from, p2p::Libp2pCommand::Publish { topic, data })
}

#[cfg(test)]
mod tests {
    use super::route_specific_room_bytes;
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
        router.route(
            &subscriber,
            p2p::Libp2pCommand::SubscribeRoom { room: room.clone() },
        );
        router.route(
            &publisher,
            p2p::Libp2pCommand::Dial(identity::Peer {
                id: subscriber.clone(),
                addrs: Vec::new(),
            }),
        );
        let events = route_specific_room_bytes(&mut router, &publisher, &room, vec![1, 2, 3]);
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: subscriber,
                event: p2p::Libp2pEvent::Gossip {
                    topic: identity::Topic::from_room(&game, &room),
                    from: publisher,
                    data: vec![1, 2, 3],
                },
            }]
        );
    }

    #[test]
    fn unsubscribed_peers_get_no_room_bytes() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let publisher = registered(&mut router, "a");
        let listener = registered(&mut router, "b");
        let room = identity::RoomName::from("room-a");
        router.route(
            &publisher,
            p2p::Libp2pCommand::Dial(identity::Peer {
                id: listener,
                addrs: Vec::new(),
            }),
        );
        let events = route_specific_room_bytes(&mut router, &publisher, &room, vec![1]);
        assert_eq!(events, Vec::new());
    }
}
