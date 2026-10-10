use libp2p::gossipsub;

use crate::net_id;
use crate::p2p;

pub fn handle_gossipsub<T: p2p::Message>(
    swarm: &libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    gossip_event: gossipsub::Event,
) {
    match gossip_event {
        gossipsub::Event::Message {
            propagation_source,
            message_id,
            message,
        } => {
            log_gossip_in(swarm, &propagation_source, &message_id, &message);
            forward_gossip(event_tx, propagation_source, message);
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
    propagation_source: libp2p::PeerId,
    message: gossipsub::Message,
) {
    event_tx
        .send(p2p::Event::Net(p2p::NetEvent::Gossip {
            topic: net_id::Topic(message.topic.to_string()),
            from: net_id::PeerId(
                message
                    .source
                    .map_or_else(|| propagation_source.to_string(), |s| s.to_string()),
            ),
            data: message.data,
        }))
        .ok();
}

#[cfg(test)]
mod tests {
    use libp2p::gossipsub;
    use libp2p::identity::Keypair;

    use super::handle_gossipsub;
    use crate::p2p;

    fn swarm() -> libp2p::Swarm<p2p::Behaviour<u32>> {
        p2p::build_swarm::<u32>(Keypair::generate_ed25519(), p2p::MdnsMode::Disabled).unwrap()
    }

    #[test]
    fn test_usage() {
        let swarm = swarm();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let relay = libp2p::PeerId::random();
        let message = gossipsub::Message {
            source: None,
            data: vec![7],
            sequence_number: None,
            topic: gossipsub::TopicHash::from_raw("lobby/topic"),
        };
        let event = gossipsub::Event::Message {
            propagation_source: relay,
            message_id: gossipsub::MessageId::new(b"m1"),
            message,
        };
        handle_gossipsub(&swarm, &tx, event);
        let Ok(p2p::Event::Net(p2p::NetEvent::Gossip { topic, from, data })) = rx.try_recv() else {
            panic!("expected a gossip event");
        };
        assert_eq!((topic.as_str(), data), ("lobby/topic", vec![7]));
        assert_eq!(from.as_str(), relay.to_string());
    }
}
