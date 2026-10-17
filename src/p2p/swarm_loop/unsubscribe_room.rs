use libp2p::gossipsub;

use crate::identity;
use crate::p2p;

pub fn unsubscribe_room<Payload: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<Payload>>,
    subscriptions: &mut p2p::swarm_loop::RoomSubscriptions,
    room: &identity::RoomName,
) {
    let topic = gossipsub::IdentTopic::new(p2p::room_topic(&subscriptions.game, room).as_str());
    let _ = swarm.behaviour_mut().gossipsub.unsubscribe(&topic);
    subscriptions.unregister(&topic);
    tracing::info!(target: "p2p", room = %room.as_str(), "p2p room unsubscribed");
}

// no test_usage necessary
