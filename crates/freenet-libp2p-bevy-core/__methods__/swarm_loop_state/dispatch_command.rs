use crate::p2p;

pub fn dispatch_command<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    config: &p2p::GossipsubConfig,
    cmd: Option<p2p::Command<Payload>>,
) -> bool {
    match cmd {
        Some(p2p::Command::Net(libp2p_command)) => {
            state.dispatch_libp2p_command(libp2p_command);
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
        Some(p2p::Command::PublishInJoinedZone { payload }) => {
            state.publish_joined(config, &payload);
        }
        Some(p2p::Command::PublishToSpecificZone { zone, payload }) => {
            state.publish_zone(config, &zone, &payload);
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
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = testing::swarm_loop_state::<Vec<u8>>().expect("swarm loop state");
        let subscribe = p2p::Command::Net(p2p::Libp2pCommand::Subscribe {
            topic: identity::Topic::from("global/topic"),
        });
        assert!(dispatch_command(
            &mut state,
            &p2p::GossipsubConfig::default(),
            Some(subscribe)
        ));
        assert_eq!(state.swarm.behaviour().gossipsub.topics().count(), 1);
        assert!(!dispatch_command(
            &mut state,
            &p2p::GossipsubConfig::default(),
            None
        ));
    }

    #[tokio::test]
    async fn subscribe_zone_registers_its_topic_and_joins_it() {
        let (mut state, _rx) = testing::swarm_loop_state::<Vec<u8>>().expect("swarm loop state");
        let zone = identity::ZoneName::from("zone-a");
        let command = p2p::Command::Net(p2p::Libp2pCommand::SubscribeZone { zone: zone.clone() });
        assert!(dispatch_command(
            &mut state,
            &p2p::GossipsubConfig::default(),
            Some(command)
        ));
        let topic = libp2p::gossipsub::IdentTopic::new(
            identity::Topic::from_zone(&state.subscriptions.game, &zone).as_str(),
        );
        assert_eq!(state.subscriptions.zone(&topic.hash()), Some(&zone));
        assert_eq!(state.subscriptions.joined, identity::Zone::Restricted(zone));
    }

    #[tokio::test]
    async fn publish_without_a_joined_zone_is_an_error() {
        let (mut state, mut rx) = testing::swarm_loop_state::<Vec<u8>>().expect("swarm loop state");
        let command = p2p::Command::PublishInJoinedZone {
            payload: Vec::new(),
        };
        assert!(dispatch_command(
            &mut state,
            &p2p::GossipsubConfig::default(),
            Some(command)
        ));
        let Ok(p2p::Event::Error(message)) = rx.try_recv() else {
            panic!("expected an error event");
        };
        assert!(message.contains("no joined zone"));
    }

    #[tokio::test]
    async fn oversize_zone_payload_is_an_error() {
        let (mut state, mut rx) = testing::swarm_loop_state::<Vec<u8>>().expect("swarm loop state");
        let command = p2p::Command::PublishToSpecificZone {
            zone: identity::ZoneName::from("zone-a"),
            payload: vec![0; 70_000],
        };
        assert!(dispatch_command(
            &mut state,
            &p2p::GossipsubConfig::default(),
            Some(command)
        ));
        let Ok(p2p::Event::Error(message)) = rx.try_recv() else {
            panic!("expected an error event");
        };
        assert!(message.contains("exceeds limit"));
    }
}
