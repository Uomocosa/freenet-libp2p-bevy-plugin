use std::collections::BTreeMap;

use crate::identity;
use crate::p2p;

pub fn track_room_command<Payload: p2p::traits::Message>(
    task: &p2p::SwarmTaskOverride<Payload>,
    command: &p2p::Libp2pCommand,
    game: &identity::GameId,
    rooms_by_topic: &mut BTreeMap<identity::Topic, identity::RoomName>,
) {
    let _ = task.orchestrator_port;
    match command {
        p2p::Libp2pCommand::SubscribeRoom { room } => {
            rooms_by_topic.insert(identity::Topic::from_room(game, room), room.clone());
        }
        p2p::Libp2pCommand::UnsubscribeRoom { room } => {
            rooms_by_topic.remove(&identity::Topic::from_room(game, room));
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::track_room_command;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let task = p2p::SwarmTaskOverride::<u8> {
            orchestrator_port: 1,
            peer: identity::Peer {
                id: identity::PeerId::from("peer"),
                addrs: Vec::new(),
            },
            marker: std::marker::PhantomData,
        };
        let mut rooms = BTreeMap::new();
        let game = identity::GameId::from("game");
        track_room_command(
            &task,
            &p2p::Libp2pCommand::SubscribeRoom {
                room: identity::RoomName::from("room-a"),
            },
            &game,
            &mut rooms,
        );
        assert_eq!(rooms.len(), 1);
    }
}
