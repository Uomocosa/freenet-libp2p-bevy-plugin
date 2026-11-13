use crate::p2p;
use libp2p::gossipsub;

pub fn handle_gossipsub<Payload: p2p::traits::Message>(
    state: &p2p::SwarmLoopState<Payload>,
    gossip_event: gossipsub::Event,
) {
    match gossip_event {
        gossipsub::Event::Message {
            propagation_source,
            message_id,
            message,
        } => {
            state.log_gossip_in(&propagation_source, &message_id, &message);
            state.forward_gossip(propagation_source, message);
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

#[cfg(test)]
mod tests {
    use libp2p::gossipsub;

    use super::handle_gossipsub;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let (state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let event = gossipsub::Event::Message {
            propagation_source: libp2p::PeerId::random(),
            message_id: gossipsub::MessageId::new(b"m1"),
            message: gossipsub::Message {
                source: None,
                data: vec![7],
                sequence_number: None,
                topic: gossipsub::TopicHash::from_raw("lobby/topic"),
            },
        };
        handle_gossipsub(&state, event);
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::Event::Libp2p(p2p::Libp2pEvent::Gossip { .. }))
        ));
    }
}
