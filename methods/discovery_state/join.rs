use crate::discovery;
use crate::identity;
use discovery::state_machine::{DiscoveryState, Output};

pub fn join(
    state: &mut DiscoveryState,
    room: identity::RoomName,
    now: discovery::UnixTime,
) -> Vec<Output> {
    let mut outputs = state.leave(now);
    state.held_rooms.remove(&room);
    state.evicted_rooms.remove(&room);
    let created_at = state
        .lobby
        .get(&room)
        .map_or(now, |record| record.created_at);
    state.room = Some(discovery::JoinedRoom {
        name: room.clone(),
        created_at,
        phase: discovery::JoinPhase::Joining { since: now },
        members: discovery::Members::new(),
    });
    outputs.extend(state.collect_room_peers_from_lobby());
    outputs.extend(state.broadcast_handshake());
    outputs.extend(state.dial_candidates(now));
    tracing::info!(target: "discovery", room = %room.as_str(), "discovery joined room");
    outputs.push(Output::Notify(discovery::Event::Joined(room)));
    outputs
}

#[cfg(test)]
mod tests {
    use super::join;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::DiscoveryState;

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let outputs = join(
            &mut state,
            identity::RoomName::from("r"),
            discovery::UnixTime::from(1),
        );
        assert!(state.room.is_some());
        assert!(outputs.iter().any(|output| matches!(
            output,
            discovery::state_machine::Output::Notify(discovery::Event::Joined(_))
        )));
    }
}
