use crate::identity;
use crate::p2p;
use crate::testing;

pub fn fan_out_peer_message(
    router: &testing::PeerMessageRouter,
    from: &identity::PeerId,
    command: p2p::Libp2pCommand,
) -> Vec<testing::RoutedEvent> {
    match command {
        p2p::Libp2pCommand::Publish { topic, data } => {
            let Some(connected) = router.connected_peers.get(from) else {
                return Vec::new();
            };
            connected
                .iter()
                .filter(|peer| *peer != from)
                .filter(|peer| {
                    router
                        .subscribed_topics
                        .get(*peer)
                        .is_some_and(|topics| topics.contains(&topic))
                })
                .map(|to| testing::RoutedEvent {
                    to: to.clone(),
                    event: p2p::Libp2pEvent::Gossip {
                        topic: topic.clone(),
                        from: from.clone(),
                        data: data.clone(),
                    },
                })
                .collect()
        }
        p2p::Libp2pCommand::PeerExchange { peer_id, data } => {
            if !router.is_connected(from, &peer_id) {
                return Vec::new();
            }
            vec![testing::RoutedEvent {
                to: peer_id,
                event: p2p::Libp2pEvent::PeerExchange {
                    from: from.clone(),
                    data,
                },
            }]
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::fan_out_peer_message;
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let from = identity::PeerId::from("a");
        let command = p2p::Libp2pCommand::PeerExchange {
            peer_id: identity::PeerId::from("ghost"),
            data: vec![3],
        };
        assert_eq!(fan_out_peer_message(&router, &from, command), Vec::new());

        let subscriber = identity::PeerId::from("b");
        let topic = identity::Topic::from("lobby/topic");
        router
            .connected_peers
            .entry(from.clone())
            .or_default()
            .insert(subscriber.clone());
        router
            .subscribed_topics
            .entry(subscriber.clone())
            .or_default()
            .insert(topic.clone());
        let publish = p2p::Libp2pCommand::Publish {
            topic: topic.clone(),
            data: vec![1],
        };
        assert_eq!(
            fan_out_peer_message(&router, &from, publish),
            vec![testing::RoutedEvent {
                to: subscriber,
                event: p2p::Libp2pEvent::Gossip {
                    topic,
                    from: from.clone(),
                    data: vec![1],
                },
            }]
        );
    }
}
