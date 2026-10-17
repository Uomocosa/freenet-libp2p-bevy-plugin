use libp2p::gossipsub;

use crate::identity;
use crate::p2p;

pub fn handle_gossipsub<T: p2p::Message>(
    swarm: &libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    subscriptions: &p2p::swarm_loop::RoomSubscriptions,
    gossip_event: gossipsub::Event,
) {
    match gossip_event {
        gossipsub::Event::Message {
            propagation_source,
            message_id,
            message,
        } => {
            log_gossip_in(swarm, &propagation_source, &message_id, &message);
            forward_gossip(event_tx, subscriptions, propagation_source, message);
        }
        gossipsub::Event::Subscribed { peer_id, topic } => {
            tracing::debug!(target: "p2p", peer = %peer_id, topic = %topic, "p2p gossip subscribed");
        }
        gossipsub::Event::Unsubscribed { peer_id, topic } => {
            tracing::debug!(target: "p2p", peer = %peer_id, topic = %topic, "p2p gossip unsubscribed");
        }
        gossipsub::Event::SlowPeer { peer_id, .. } => {
            tracing::warn!(target: "p2p", peer = %peer_id, "p2p gossip slow peer");
        }
        gossipsub::Event::GossipsubNotSupported { peer_id } => {
            tracing::warn!(target: "p2p", peer = %peer_id, "p2p gossip not supported");
        }
    }
}

// needed helper: samples gossip mesh depth for one inbound message
fn log_gossip_in<T: p2p::Message>(
    swarm: &libp2p::Swarm<p2p::Behaviour<T>>,
    propagation_source: &libp2p::PeerId,
    message_id: &libp2p::gossipsub::MessageId,
    message: &gossipsub::Message,
) {
    let mesh = swarm
        .behaviour()
        .gossipsub
        .mesh_peers(&message.topic)
        .count();
    tracing::debug!(
        target: "p2p",
        source = %propagation_source,
        message = %message_id,
        topic = %message.topic,
        mesh,
        "p2p gossip received"
    );
}

// needed helper: converts a gossipsub wire message into a Bevy event
fn forward_gossip<T: p2p::Message>(
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    subscriptions: &p2p::swarm_loop::RoomSubscriptions,
    propagation_source: libp2p::PeerId,
    message: gossipsub::Message,
) {
    let from = identity::PeerId(message.source.unwrap_or(propagation_source).to_string());
    let Some(room) = subscriptions.room(&message.topic) else {
        event_tx
            .send(p2p::Event::Net(p2p::NetEvent::Gossip {
                topic: identity::Topic(message.topic.to_string()),
                from,
                data: message.data,
            }))
            .ok();
        return;
    };
    match bincode::deserialize::<T>(&message.data) {
        Ok(payload) => {
            event_tx
                .send(p2p::Event::RoomMessage {
                    room: room.clone(),
                    from,
                    payload,
                })
                .ok();
        }
        Err(error) => {
            event_tx
                .send(p2p::Event::Error(format!(
                    "room payload decode failed: {error}"
                )))
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use libp2p::gossipsub;
    use libp2p::identity::Keypair;

    use super::handle_gossipsub;
    use crate::identity;
    use crate::p2p;

    fn swarm() -> libp2p::Swarm<p2p::Behaviour<u32>> {
        p2p::build_swarm::<u32>(
            Keypair::generate_ed25519(),
            p2p::MdnsMode::Disabled,
            &identity::GameId::from("test"),
            &p2p::GossipsubConfig::default(),
        )
        .unwrap()
    }

    fn subscriptions() -> p2p::swarm_loop::RoomSubscriptions {
        p2p::swarm_loop::RoomSubscriptions::new(identity::GameId::from("game"))
    }

    fn message_event(
        propagation_source: &libp2p::PeerId,
        source: Option<libp2p::PeerId>,
        data: Vec<u8>,
        topic: gossipsub::TopicHash,
    ) -> gossipsub::Event {
        gossipsub::Event::Message {
            propagation_source: *propagation_source,
            message_id: gossipsub::MessageId::new(b"m1"),
            message: gossipsub::Message {
                source,
                data,
                sequence_number: None,
                topic,
            },
        }
    }

    #[test]
    fn test_usage() {
        let swarm = swarm();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let subscriptions = subscriptions();
        let relay = libp2p::PeerId::random();
        let event = message_event(
            &relay,
            None,
            vec![7],
            gossipsub::TopicHash::from_raw("lobby/topic"),
        );
        handle_gossipsub(&swarm, &tx, &subscriptions, event);
        let Ok(p2p::Event::Net(p2p::NetEvent::Gossip { topic, from, data })) = rx.try_recv() else {
            panic!("expected a gossip event");
        };
        assert_eq!((topic.as_str(), data), ("lobby/topic", vec![7]));
        assert_eq!(from.as_str(), relay.to_string());
    }

    #[test]
    fn registered_room_topic_becomes_a_room_message() {
        let swarm = swarm();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut subscriptions = subscriptions();
        let room = identity::RoomName::from("room-a");
        let topic =
            gossipsub::IdentTopic::new(p2p::room_topic(&subscriptions.game, &room).as_str());
        subscriptions.register(&topic, room.clone());
        let data = bincode::serialize(&9u32).unwrap();
        handle_gossipsub(
            &swarm,
            &tx,
            &subscriptions,
            message_event(
                &libp2p::PeerId::random(),
                Some(libp2p::PeerId::random()),
                data,
                topic.hash(),
            ),
        );
        let Ok(p2p::Event::RoomMessage {
            room: got, payload, ..
        }) = rx.try_recv()
        else {
            panic!("expected a room message");
        };
        assert_eq!((got, payload), (room, 9));
    }

    #[test]
    fn undecodable_room_payload_is_an_error() {
        let swarm = swarm();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut subscriptions = subscriptions();
        let room = identity::RoomName::from("room-a");
        let topic =
            gossipsub::IdentTopic::new(p2p::room_topic(&subscriptions.game, &room).as_str());
        subscriptions.register(&topic, room);
        handle_gossipsub(
            &swarm,
            &tx,
            &subscriptions,
            message_event(&libp2p::PeerId::random(), None, Vec::new(), topic.hash()),
        );
        let Ok(p2p::Event::Error(message)) = rx.try_recv() else {
            panic!("expected an error event");
        };
        assert!(message.contains("decode failed"));
    }
}
