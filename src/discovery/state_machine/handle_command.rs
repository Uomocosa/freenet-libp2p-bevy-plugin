use crate::discovery;
use crate::net_id;
use discovery::state_machine::{Output, State};

pub fn handle_command(
    state: &mut State,
    command: discovery::Command,
    now: discovery::UnixTime,
) -> Vec<Output> {
    match command {
        discovery::Command::Create(room) | discovery::Command::Join(room) => join(state, room, now),
        discovery::Command::Leave => leave(state),
    }
}

// needed helper: enters a room, says hello to every live link and dials the lobby seeds
fn join(state: &mut State, room: net_id::RoomName, now: discovery::UnixTime) -> Vec<Output> {
    let mut outputs = leave(state);
    state.room = Some(discovery::Room {
        name: room.clone(),
        members: discovery::Members::new(),
    });
    discovery::state_machine::seed_candidates(state);
    outputs.extend(discovery::state_machine::broadcast_hello(state));
    outputs.extend(discovery::state_machine::dial_candidates(state, now));
    tracing::info!(target: "room_lobby", room = %room.as_str(), "discovery joined room");
    outputs.push(Output::Notify(discovery::Event::Joined(room)));
    outputs
}

// needed helper: leaves the current room and tells every live link
fn leave(state: &mut State) -> Vec<Output> {
    let Some(room) = state.room.take() else {
        return Vec::new();
    };
    state.candidates.clear();
    state.last_dial.clear();
    let mut outputs = discovery::state_machine::broadcast_hello(state);
    outputs.push(Output::Notify(discovery::Event::Left(room.name)));
    outputs
}

#[cfg(test)]
mod tests {
    use super::handle_command;
    use crate::discovery;
    use crate::net_id;
    use crate::p2p;
    use discovery::state_machine::{Output, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        state.connected.insert(net_id::PeerId::from("a"));
        let room = net_id::RoomName::from("r");
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
                Output::NetCommand(p2p::NetCommand::Exchange { .. }),
                Output::Notify(discovery::Event::Joined(_)),
                Output::NetCommand(p2p::NetCommand::Exchange { .. }),
                Output::Notify(discovery::Event::Left(_)),
            ]
        ));
    }
}
