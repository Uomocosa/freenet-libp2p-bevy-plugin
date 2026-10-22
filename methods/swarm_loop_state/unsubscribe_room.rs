use libp2p::gossipsub;

use crate::identity;
use crate::p2p;

pub fn unsubscribe_room<Payload: p2p::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    room: &identity::RoomName,
) {
    let topic = gossipsub::IdentTopic::new(
        identity::Topic::for_room(&state.subscriptions.game, room).as_str(),
    );
    let _ = state.swarm.behaviour_mut().gossipsub.unsubscribe(&topic);
    state.subscriptions.unregister(&topic);
    tracing::info!(target: "p2p", room = %room.as_str(), "p2p room unsubscribed");
}

#[cfg(test)]
mod tests {
    use super::unsubscribe_room;

    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        let room = identity::RoomName::from("room-a");
        state.subscribe_room(room.clone());
        unsubscribe_room(&mut state, &room);
        assert!(state.subscriptions.joined.is_none());
    }
}
