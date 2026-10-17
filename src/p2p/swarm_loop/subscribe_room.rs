use libp2p::gossipsub;

use crate::identity;
use crate::p2p;

pub fn subscribe_room<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    rooms: &mut p2p::swarm_loop::RoomLoop,
    room: identity::RoomName,
) {
    let topic = gossipsub::IdentTopic::new(p2p::room_topic(&rooms.topics.game, &room).as_str());
    match swarm.behaviour_mut().gossipsub.subscribe(&topic) {
        Ok(_) => {
            tracing::info!(target: "p2p", room = %room.as_str(), "p2p room subscribed");
            rooms.topics.register(&topic, room);
        }
        Err(error) => {
            event_tx
                .send(p2p::Event::Error(format!(
                    "room subscribe {}: {error}",
                    room.as_str()
                )))
                .ok();
        }
    }
}

// no test_usage necessary
