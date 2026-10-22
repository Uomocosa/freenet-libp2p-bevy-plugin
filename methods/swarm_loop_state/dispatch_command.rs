use crate::identity;
use crate::p2p;

pub fn dispatch_command<Payload: p2p::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    config: &p2p::GossipsubConfig,
    cmd: Option<p2p::Command<Payload>>,
) -> bool {
    match cmd {
        Some(p2p::Command::Net(net)) => {
            state.dispatch_net_command(net);
        }
        Some(p2p::Command::Send { peer_id, payload }) => {
            if let Ok(libp2p_id) = peer_id.parse::<libp2p::PeerId>() {
                state
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&libp2p_id, payload);
            }
        }
        Some(p2p::Command::PublishInJoinedRoom { payload }) => {
            publish_joined(state, config, &payload);
        }
        Some(p2p::Command::PublishToSpecificRoom { room, payload }) => {
            publish_room(state, config, &room, &payload);
        }
        None => return false,
    }
    true
}

// needed helper: publishes to the joined room, or reports that there is none
fn publish_joined<Payload: p2p::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    config: &p2p::GossipsubConfig,
    payload: &Payload,
) {
    let Some(room) = state.subscriptions.joined.clone() else {
        state
            .event_tx
            .send(p2p::Event::Error(
                "room publish: no joined room".to_string(),
            ))
            .ok();
        return;
    };
    publish_room(state, config, &room, payload);
}

// needed helper: serializes one room publish and reports every rejection
fn publish_room<Payload: p2p::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    config: &p2p::GossipsubConfig,
    room: &identity::RoomName,
    payload: &Payload,
) {
    let Ok(data) = bincode::serialize(payload) else {
        state
            .event_tx
            .send(p2p::Event::Error(format!(
                "room publish {}: encode failed",
                room.as_str()
            )))
            .ok();
        return;
    };
    if data.len() > config.max_transmit_size {
        state
            .event_tx
            .send(p2p::Event::Error(format!(
                "room publish {}: payload {} bytes exceeds limit {}",
                room.as_str(),
                data.len(),
                config.max_transmit_size
            )))
            .ok();
        return;
    }
    let topic = libp2p::gossipsub::IdentTopic::new(
        identity::Topic::for_room(&state.subscriptions.game, room).as_str(),
    );
    if let Err(error) = state.swarm.behaviour_mut().gossipsub.publish(topic, data) {
        state
            .event_tx
            .send(p2p::Event::Error(format!(
                "room publish {}: {error}",
                room.as_str()
            )))
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use super::dispatch_command;

    use crate::identity;
    use crate::p2p;

    fn config() -> p2p::GossipsubConfig {
        p2p::GossipsubConfig::default()
    }

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = p2p::SwarmLoopState::<Vec<u8>>::for_test();
        let subscribe = p2p::Command::Net(p2p::NetCommand::Subscribe {
            topic: identity::Topic::from("lobby/topic"),
        });
        assert!(dispatch_command(&mut state, &config(), Some(subscribe)));
        assert_eq!(state.swarm.behaviour().gossipsub.topics().count(), 1);
        assert!(!dispatch_command(&mut state, &config(), None));
    }

    #[tokio::test]
    async fn subscribe_room_registers_its_topic_and_joins_it() {
        let (mut state, _rx) = p2p::SwarmLoopState::<Vec<u8>>::for_test();
        let room = identity::RoomName::from("room-a");
        let command = p2p::Command::Net(p2p::NetCommand::SubscribeRoom { room: room.clone() });
        assert!(dispatch_command(&mut state, &config(), Some(command)));
        let topic = libp2p::gossipsub::IdentTopic::new(
            identity::Topic::for_room(&state.subscriptions.game, &room).as_str(),
        );
        assert_eq!(state.subscriptions.room(&topic.hash()), Some(&room));
        assert_eq!(state.subscriptions.joined, Some(room));
    }

    #[tokio::test]
    async fn publish_without_a_joined_room_is_an_error() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<Vec<u8>>::for_test();
        let command = p2p::Command::PublishInJoinedRoom {
            payload: Vec::new(),
        };
        assert!(dispatch_command(&mut state, &config(), Some(command)));
        let Ok(p2p::Event::Error(message)) = rx.try_recv() else {
            panic!("expected an error event");
        };
        assert!(message.contains("no joined room"));
    }

    #[tokio::test]
    async fn oversize_room_payload_is_an_error() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<Vec<u8>>::for_test();
        let command = p2p::Command::PublishToSpecificRoom {
            room: identity::RoomName::from("room-a"),
            payload: vec![0; 70_000],
        };
        assert!(dispatch_command(&mut state, &config(), Some(command)));
        let Ok(p2p::Event::Error(message)) = rx.try_recv() else {
            panic!("expected an error event");
        };
        assert!(message.contains("exceeds limit"));
    }
}
