use std::collections::BTreeMap;

use crate::identity;
use crate::p2p;

pub fn handle_server_event<Payload: p2p::Message>(
    task: &p2p::SwarmTaskOverride<Payload>,
    rooms_by_topic: &BTreeMap<identity::Topic, identity::RoomName>,
    event: p2p::Libp2pEvent,
    events: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
) {
    let _ = task.orchestrator_port;
    if let p2p::Libp2pEvent::Gossip { topic, from, data } = event {
        if let Some(room) = rooms_by_topic.get(&topic) {
            match bincode::deserialize::<Payload>(&data) {
                Ok(payload) => {
                    events
                        .send(p2p::Event::RoomMessage {
                            room: room.clone(),
                            from,
                            payload,
                        })
                        .ok();
                }
                Err(error) => {
                    events
                        .send(p2p::Event::Error(format!(
                            "room payload decode failed: {error}"
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
    use std::collections::BTreeMap;

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
            &BTreeMap::new(),
            p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("peer")),
            &event_tx,
        );
    }
}
