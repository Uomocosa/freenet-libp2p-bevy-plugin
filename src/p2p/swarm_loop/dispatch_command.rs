use crate::identity;
use crate::p2p;

pub fn dispatch_command<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    rooms: &mut p2p::swarm_loop::RoomLoop,
    cmd: Option<p2p::Command<T>>,
) -> bool {
    match cmd {
        Some(p2p::Command::Net(net)) => {
            p2p::swarm_loop::dispatch_net_command(swarm, event_tx, rooms, net);
        }
        Some(p2p::Command::Send { peer_id, payload }) => {
            if let Ok(pid) = peer_id.parse::<libp2p::PeerId>() {
                swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&pid, payload);
            }
        }
        Some(p2p::Command::Publish { payload }) => {
            publish_joined(swarm, event_tx, rooms, &payload);
        }
        Some(p2p::Command::PublishTo { room, payload }) => {
            publish_room(swarm, event_tx, rooms, &room, &payload);
        }
        None => return false,
    }
    true
}

// needed helper: publishes to the joined room, or reports that there is none
fn publish_joined<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    rooms: &p2p::swarm_loop::RoomLoop,
    payload: &T,
) {
    let Some(room) = rooms.topics.joined.as_ref() else {
        event_tx
            .send(p2p::Event::Error(
                "room publish: no joined room".to_string(),
            ))
            .ok();
        return;
    };
    publish_room(swarm, event_tx, rooms, room, payload);
}

// needed helper: serializes one room publish and reports every rejection
fn publish_room<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    rooms: &p2p::swarm_loop::RoomLoop,
    room: &identity::RoomName,
    payload: &T,
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
    if data.len() > rooms.config.gossipsub.max_transmit_size {
        event_tx
            .send(p2p::Event::Error(format!(
                "room publish {}: payload {} bytes exceeds limit {}",
                room.as_str(),
                data.len(),
                rooms.config.gossipsub.max_transmit_size
            )))
            .ok();
        return;
    }
    let topic =
        libp2p::gossipsub::IdentTopic::new(p2p::room_topic(&rooms.topics.game, room).as_str());
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

    fn rooms() -> p2p::swarm_loop::RoomLoop {
        p2p::swarm_loop::RoomLoop::new(identity::GameId::from("game"), p2p::RoomConfig::default())
    }

    #[tokio::test]
    async fn test_usage() {
        let mut swarm = swarm();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut rooms = rooms();
        let subscribe = p2p::Command::Net(p2p::NetCommand::Subscribe {
            topic: identity::Topic::from("lobby/topic"),
        });
        assert!(dispatch_command(
            &mut swarm,
            &tx,
            &mut rooms,
            Some(subscribe)
        ));
        assert_eq!(swarm.behaviour().gossipsub.topics().count(), 1);
        assert!(!dispatch_command(&mut swarm, &tx, &mut rooms, None));
    }

    #[tokio::test]
    async fn subscribe_room_registers_its_topic_and_joins_it() {
        let mut swarm = swarm();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut rooms = rooms();
        let room = identity::RoomName::from("room-a");
        let command = p2p::Command::Net(p2p::NetCommand::SubscribeRoom { room: room.clone() });
        assert!(dispatch_command(&mut swarm, &tx, &mut rooms, Some(command)));
        let topic =
            libp2p::gossipsub::IdentTopic::new(p2p::room_topic(&rooms.topics.game, &room).as_str());
        assert_eq!(rooms.topics.room(&topic.hash()), Some(&room));
        assert_eq!(rooms.topics.joined, Some(room));
    }

    #[tokio::test]
    async fn publish_without_a_joined_room_is_an_error() {
        let mut swarm = swarm();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut rooms = rooms();
        let command = p2p::Command::Publish {
            payload: Vec::new(),
        };
        assert!(dispatch_command(&mut swarm, &tx, &mut rooms, Some(command)));
        let Ok(p2p::Event::Error(message)) = rx.try_recv() else {
            panic!("expected an error event");
        };
        assert!(message.contains("no joined room"));
    }

    #[tokio::test]
    async fn oversize_room_payload_is_an_error() {
        let mut swarm = swarm();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut rooms = rooms();
        let command = p2p::Command::PublishTo {
            room: identity::RoomName::from("room-a"),
            payload: vec![0; 70_000],
        };
        assert!(dispatch_command(&mut swarm, &tx, &mut rooms, Some(command)));
        let Ok(p2p::Event::Error(message)) = rx.try_recv() else {
            panic!("expected an error event");
        };
        assert!(message.contains("exceeds limit"));
    }
}
