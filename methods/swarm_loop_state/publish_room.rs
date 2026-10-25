use crate::identity;
use crate::p2p;

pub fn publish_room<Payload: p2p::Message>(
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
        identity::Topic::from_room(&state.subscriptions.game, room).as_str(),
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
    use super::publish_room;
    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<Vec<u8>>::for_test();
        let config = p2p::GossipsubConfig::default();
        let room = identity::RoomName::from("room-a");
        publish_room(&mut state, &config, &room, &vec![0; 70_000]);
        let Ok(p2p::Event::Error(message)) = rx.try_recv() else {
            panic!("expected an error event");
        };
        assert!(message.contains("exceeds limit"));
    }
}
