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
            state.publish_joined(config, &payload);
        }
        Some(p2p::Command::PublishToSpecificRoom { room, payload }) => {
            state.publish_room(config, &room, &payload);
        }
        None => return false,
    }
    true
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
            identity::Topic::from_room(&state.subscriptions.game, &room).as_str(),
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
