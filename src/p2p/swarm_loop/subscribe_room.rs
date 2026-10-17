use libp2p::gossipsub;

use crate::identity;
use crate::p2p;

pub fn subscribe_room<Payload: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<Payload>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    subscriptions: &mut p2p::swarm_loop::RoomSubscriptions,
    room: identity::RoomName,
) {
    let topic = gossipsub::IdentTopic::new(p2p::room_topic(&subscriptions.game, &room).as_str());
    match swarm.behaviour_mut().gossipsub.subscribe(&topic) {
        Ok(_) => {
            tracing::info!(target: "p2p", room = %room.as_str(), "p2p room subscribed");
            subscriptions.register(&topic, room);
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
