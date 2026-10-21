use crate::discovery;
use crate::identity;
use discovery::state_machine::{Output, State};

pub fn handle_command(
    state: &mut State,
    command: discovery::Command,
    now: discovery::UnixTime,
) -> Vec<Output> {
    match command {
        discovery::Command::Create(room) | discovery::Command::Join(room) => {
            if room.as_str().trim().is_empty() {
                tracing::error!(target: "discovery", "discovery rejected: empty room name");
                Vec::new()
            } else {
                join(state, room, now)
            }
        }
        discovery::Command::Leave => leave(state, now),
        discovery::Command::Quit => quit(state, now),
    }
}

// needed helper: leaves, then marks every held room so the next publish is a Quit tombstone
fn quit(state: &mut State, now: discovery::UnixTime) -> Vec<Output> {
    let outputs = leave(state, now);
    state.is_quitting = true;
    outputs
}

// needed helper: enters a room, says handshake to every live link and dials the lobby seeds
fn join(state: &mut State, room: identity::RoomName, now: discovery::UnixTime) -> Vec<Output> {
    let mut outputs = leave(state, now);
    state.held_rooms.remove(&room);
    state.evicted_rooms.remove(&room);
    let created_at = state
        .lobby
        .get(&room)
        .map_or(now, |record| record.created_at);
    state.room = Some(discovery::JoinedRoom {
        name: room.clone(),
        created_at,
        phase: discovery::RoomPhase::Joining,
        members: discovery::Members::new(),
    });
    state.joining_since = Some(now);
    outputs.extend(discovery::state_machine::collect_room_peers_from_lobby(
        state,
    ));
    outputs.extend(discovery::state_machine::broadcast_handshake(state));
    outputs.extend(discovery::state_machine::dial_candidates(state, now));
    tracing::info!(target: "discovery", room = %room.as_str(), "discovery joined room");
    outputs.push(Output::Notify(discovery::Event::Joined(room)));
    outputs
}

// needed helper: leaves the current room and tells every live link
fn leave(state: &mut State, now: discovery::UnixTime) -> Vec<Output> {
    let Some(room) = state.room.take() else {
        return Vec::new();
    };
    state.held_rooms.insert(
        room.name.clone(),
        discovery::state_machine::HeldRoomHistory {
            created_at: room.created_at,
            left_at: now,
        },
    );
    state.joining_since = None;
    state.candidates.clear();
    state.last_dialed_at.clear();
    let mut outputs = discovery::state_machine::broadcast_handshake(state);
    outputs.push(Output::Notify(discovery::Event::Left(room.name)));
    outputs
}

#[cfg(test)]
mod tests {
    use super::handle_command;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use discovery::state_machine::{Output, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.connected.insert(identity::PeerId::from("a"));
        let room = identity::RoomName::from("r");
        let mut outputs = handle_command(
            &mut state,
            discovery::Command::Create(room),
            discovery::UnixTime::from_secs(1),
        );
        assert!(state.room.is_some());
        outputs.extend(handle_command(
            &mut state,
            discovery::Command::Leave,
            discovery::UnixTime::from_secs(2),
        ));
        assert!(state.room.is_none());
        assert!(matches!(
            outputs.as_slice(),
            [
                Output::NetCommand(p2p::NetCommand::PeerExchange { .. }),
                Output::Notify(discovery::Event::Joined(_)),
                Output::NetCommand(p2p::NetCommand::PeerExchange { .. }),
                Output::Notify(discovery::Event::Left(_)),
            ]
        ));
    }

    #[test]
    fn test_empty_room_rejected() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let outputs = handle_command(
            &mut state,
            discovery::Command::Create(identity::RoomName::from("   ")),
            discovery::UnixTime::from_secs(1),
        );
        assert_eq!(outputs.len(), 0);
        assert!(state.room.is_none());
    }
}
