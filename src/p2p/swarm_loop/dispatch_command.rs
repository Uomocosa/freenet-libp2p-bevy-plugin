use crate::identity;
use crate::p2p;

pub fn dispatch_command<Payload: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<Payload>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    subscriptions: &mut p2p::swarm_loop::RoomSubscriptions,
    snapshot_replies: &mut p2p::swarm_loop::PendingSnapshotReplies,
    config: &p2p::GossipsubConfig,
    cmd: Option<p2p::Command<Payload>>,
) -> bool {
    match cmd {
        Some(p2p::Command::Net(net)) => {
            p2p::swarm_loop::dispatch_net_command(
                swarm,
                event_tx,
                subscriptions,
                snapshot_replies,
                net,
            );
        }
        Some(p2p::Command::Send { peer_id, payload }) => {
            if let Ok(libp2p_id) = peer_id.parse::<libp2p::PeerId>() {
                swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&libp2p_id, payload);
            }
        }
        Some(p2p::Command::PublishInJoinedRoom { payload }) => {
            publish_joined(swarm, event_tx, subscriptions, config, &payload);
        }
        Some(p2p::Command::PublishToSpecificRoom { room, payload }) => {
            publish_room(swarm, event_tx, subscriptions, config, &room, &payload);
        }
        None => return false,
    }
    true
}

// needed helper: publishes to the joined room, or reports that there is none
fn publish_joined<Payload: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<Payload>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    subscriptions: &p2p::swarm_loop::RoomSubscriptions,
    config: &p2p::GossipsubConfig,
    payload: &Payload,
) {
    let Some(room) = subscriptions.joined.as_ref() else {
        event_tx
            .send(p2p::Event::Error(
                "room publish: no joined room".to_string(),
            ))
            .ok();
        return;
    };
    publish_room(swarm, event_tx, subscriptions, config, room, payload);
}

// needed helper: serializes one room publish and reports every rejection
fn publish_room<Payload: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<Payload>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    subscriptions: &p2p::swarm_loop::RoomSubscriptions,
    config: &p2p::GossipsubConfig,
    room: &identity::RoomName,
    payload: &Payload,
) {
    let Ok(data) = bincode::serialize(payload) else {
        event_tx
            .send(p2p::Event::Error(format!(
                "room publish {}: encode failed",
                room.as_str()
            )))
            .ok();
        return;
    };
    if data.len() > config.max_transmit_size {
        event_tx
            .send(p2p::Event::Error(format!(
                "room publish {}: payload {} bytes exceeds limit {}",
                room.as_str(),
                data.len(),
                config.max_transmit_size
            )))
            .ok();
        return;
    }
    let topic =
        libp2p::gossipsub::IdentTopic::new(p2p::room_topic(&subscriptions.game, room).as_str());
    if let Err(error) = swarm.behaviour_mut().gossipsub.publish(topic, data) {
        event_tx
            .send(p2p::Event::Error(format!(
                "room publish {}: {error}",
                room.as_str()
            )))
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use libp2p::identity::Keypair;

    use super::dispatch_command;
    use crate::identity;
    use crate::p2p;

    fn swarm() -> libp2p::Swarm<p2p::Behaviour<Vec<u8>>> {
        p2p::build_swarm::<Vec<u8>>(
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

    fn snapshot_replies() -> p2p::swarm_loop::PendingSnapshotReplies {
        p2p::swarm_loop::PendingSnapshotReplies::default()
    }

    fn config() -> p2p::GossipsubConfig {
        p2p::GossipsubConfig::default()
    }

    #[tokio::test]
    async fn test_usage() {
        let mut swarm = swarm();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut subscriptions = subscriptions();
        let subscribe = p2p::Command::Net(p2p::NetCommand::Subscribe {
            topic: identity::Topic::from("lobby/topic"),
        });
        assert!(dispatch_command(
            &mut swarm,
            &tx,
            &mut subscriptions,
            &mut snapshot_replies(),
            &config(),
            Some(subscribe)
        ));
        assert_eq!(swarm.behaviour().gossipsub.topics().count(), 1);
        assert!(!dispatch_command(
            &mut swarm,
            &tx,
            &mut subscriptions,
            &mut snapshot_replies(),
            &config(),
            None
        ));
    }

    #[tokio::test]
    async fn subscribe_room_registers_its_topic_and_joins_it() {
        let mut swarm = swarm();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut subscriptions = subscriptions();
        let room = identity::RoomName::from("room-a");
        let command = p2p::Command::Net(p2p::NetCommand::SubscribeRoom { room: room.clone() });
        assert!(dispatch_command(
            &mut swarm,
            &tx,
            &mut subscriptions,
            &mut snapshot_replies(),
            &config(),
            Some(command)
        ));
        let topic = libp2p::gossipsub::IdentTopic::new(
            p2p::room_topic(&subscriptions.game, &room).as_str(),
        );
        assert_eq!(subscriptions.room(&topic.hash()), Some(&room));
        assert_eq!(subscriptions.joined, Some(room));
    }

    #[tokio::test]
    async fn publish_without_a_joined_room_is_an_error() {
        let mut swarm = swarm();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut subscriptions = subscriptions();
        let command = p2p::Command::PublishInJoinedRoom {
            payload: Vec::new(),
        };
        assert!(dispatch_command(
            &mut swarm,
            &tx,
            &mut subscriptions,
            &mut snapshot_replies(),
            &config(),
            Some(command)
        ));
        let Ok(p2p::Event::Error(message)) = rx.try_recv() else {
            panic!("expected an error event");
        };
        assert!(message.contains("no joined room"));
    }

    #[tokio::test]
    async fn oversize_room_payload_is_an_error() {
        let mut swarm = swarm();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut subscriptions = subscriptions();
        let command = p2p::Command::PublishToSpecificRoom {
            room: identity::RoomName::from("room-a"),
            payload: vec![0; 70_000],
        };
        assert!(dispatch_command(
            &mut swarm,
            &tx,
            &mut subscriptions,
            &mut snapshot_replies(),
            &config(),
            Some(command)
        ));
        let Ok(p2p::Event::Error(message)) = rx.try_recv() else {
            panic!("expected an error event");
        };
        assert!(message.contains("exceeds limit"));
    }
}
