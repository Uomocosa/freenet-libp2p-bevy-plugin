use libp2p::gossipsub;

use crate::p2p;

pub fn handle_server_event<Payload: p2p::traits::Message>(
    task: &p2p::SwarmTaskOverride<Payload>,
    subscriptions: &p2p::swarm_loop::ZoneSubscriptions,
    event: p2p::Libp2pEvent,
    events: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
) {
    let _ = task.orchestrator_port;
    if let p2p::Libp2pEvent::Gossip { topic, from, data } = event {
        let hash = gossipsub::IdentTopic::new(topic.as_str()).hash();
        if let Some(zone) = subscriptions.zone(&hash) {
            match bincode::deserialize::<Payload>(&data) {
                Ok(payload) => {
                    events
                        .send(p2p::Event::ZoneMessage {
                            zone: zone.clone(),
                            from,
                            payload,
                        })
                        .ok();
                }
                Err(error) => {
                    events
                        .send(p2p::Event::Error(format!(
                            "zone payload decode failed: {error}"
                        )))
                        .ok();
                }
            }
        } else {
            events
                .send(p2p::Event::Libp2p(p2p::Libp2pEvent::Gossip {
                    topic,
                    from,
                    data,
                }))
                .ok();
        }
    } else {
        events.send(p2p::Event::Libp2p(event)).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::handle_server_event;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let task = p2p::SwarmTaskOverride::<u8> {
            orchestrator_port: 1,
            peer: identity::Peer {
                id: identity::PeerId::from("peer"),
                addrs: Vec::new(),
            },
            marker: std::marker::PhantomData,
        };
        let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel();
        handle_server_event(
            &task,
            &p2p::swarm_loop::ZoneSubscriptions::new(identity::GameId::from("game")),
            p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("peer")),
            &event_tx,
        );
    }
}
