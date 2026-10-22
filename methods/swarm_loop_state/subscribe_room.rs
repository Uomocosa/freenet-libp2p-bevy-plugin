use libp2p::gossipsub;

use crate::identity;
use crate::p2p;

pub fn subscribe_room<Payload: p2p::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    room: identity::RoomName,
) {
    let topic = gossipsub::IdentTopic::new(
        identity::Topic::for_room(&state.subscriptions.game, &room).as_str(),
    );
    match state.swarm.behaviour_mut().gossipsub.subscribe(&topic) {
        Ok(_) => {
            tracing::info!(target: "p2p", room = %room.as_str(), "p2p room subscribed");
            state.subscriptions.register(&topic, room);
        }
        Err(error) => {
            state
                .event_tx
                .send(p2p::Event::Error(format!(
                    "room subscribe {}: {error}",
                    room.as_str()
                )))
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::subscribe_room;

    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        subscribe_room(&mut state, identity::RoomName::from("room-a"));
        assert!(state.subscriptions.joined.is_some());
    }
}
