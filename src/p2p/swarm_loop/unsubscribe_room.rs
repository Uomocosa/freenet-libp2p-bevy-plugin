use libp2p::gossipsub;

use crate::identity;
use crate::p2p;

pub fn unsubscribe_room<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    rooms: &mut p2p::swarm_loop::RoomLoop,
    room: &identity::RoomName,
) {
    let topic = gossipsub::IdentTopic::new(p2p::room_topic(&rooms.topics.game, room).as_str());
    let _ = swarm.behaviour_mut().gossipsub.unsubscribe(&topic);
    rooms.topics.unregister(&topic);
    tracing::info!(target: "p2p", room = %room.as_str(), "p2p room unsubscribed");
}

// no test_usage necessary
