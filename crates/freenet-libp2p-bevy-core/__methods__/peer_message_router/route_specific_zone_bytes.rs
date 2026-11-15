use crate::identity;
use crate::p2p;
use crate::testing;

pub fn route_specific_zone_bytes(
    router: &mut testing::PeerMessageRouter,
    from: &identity::PeerId,
    zone: &identity::ZoneName,
    data: Vec<u8>,
) -> Vec<testing::RoutedEvent> {
    let topic = identity::Topic::from_zone(&router.game, zone);
    router.route(from, p2p::Libp2pCommand::Publish { topic, data })
}

#[cfg(test)]
mod tests {
    use super::route_specific_zone_bytes;
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
        let zone = identity::ZoneName::from("zone-a");
        router.route(
            &subscriber,
            p2p::Libp2pCommand::SubscribeZone { zone: zone.clone() },
        );
        router.route(
            &publisher,
            p2p::Libp2pCommand::Dial(identity::Peer {
                id: subscriber.clone(),
                addrs: Vec::new(),
            }),
        );
        let events = route_specific_zone_bytes(&mut router, &publisher, &zone, vec![1, 2, 3]);
        assert_eq!(
            events,
            vec![testing::RoutedEvent {
                to: subscriber,
                event: p2p::Libp2pEvent::Gossip {
                    topic: identity::Topic::from_zone(&game, &zone),
                    from: publisher,
                    data: vec![1, 2, 3],
                },
            }]
        );
    }

    #[test]
    fn unsubscribed_peers_get_no_zone_bytes() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let publisher = registered(&mut router, "a");
        let listener = registered(&mut router, "b");
        let zone = identity::ZoneName::from("zone-a");
        router.route(
            &publisher,
            p2p::Libp2pCommand::Dial(identity::Peer {
                id: listener,
                addrs: Vec::new(),
            }),
        );
        let events = route_specific_zone_bytes(&mut router, &publisher, &zone, vec![1]);
        assert_eq!(events, Vec::new());
    }
}
