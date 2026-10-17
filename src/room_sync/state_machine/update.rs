use crate::room_sync;

pub fn update<Op>(
    state: &mut room_sync::state_machine::State<Op>,
    input: room_sync::state_machine::Input<Op>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    match input {
        room_sync::state_machine::Input::Joined { room, members } => {
            room_sync::state_machine::handle_joined(state, room, members)
        }
        room_sync::state_machine::Input::Members { members } => {
            room_sync::state_machine::handle_members(state, members)
        }
        room_sync::state_machine::Input::Remote { room, meta, op } => {
            room_sync::state_machine::handle_remote(state, &room, meta, op)
        }
        room_sync::state_machine::Input::Snapshot {
            room,
            part,
            total,
            data,
        } => room_sync::state_machine::handle_snapshot(state, room, part, total, data),
        room_sync::state_machine::Input::Fetch { id, room, part } => {
            room_sync::state_machine::handle_fetch(state, id, room, part)
        }
        room_sync::state_machine::Input::Tick => room_sync::state_machine::tick(state),
        room_sync::state_machine::Input::Failed => room_sync::state_machine::retry_fetch(state),
    }
}

#[cfg(test)]
mod tests {
    use super::update;
    use crate::identity;
    use crate::room_sync;
    use crate::room_sync::state_machine::{Input, Output, Phase, State};

    fn meta(lamport: u32) -> room_sync::OpMeta {
        room_sync::OpMeta { lamport, author: 1 }
    }

    #[test]
    fn test_usage() {
        let mut state = State::<u8>::default();
        let room = identity::RoomName::from("r");
        let outputs = update(
            &mut state,
            Input::Joined {
                room: room.clone(),
                members: vec![identity::PeerId::from("a")],
            },
        );
        assert_eq!(state.phase, Phase::Fetching);
        assert_eq!(
            outputs,
            vec![Output::Fetch {
                peer: identity::PeerId::from("a"),
                room: room.clone(),
                part: 0,
            }]
        );
        update(
            &mut state,
            Input::Snapshot {
                room: room.clone(),
                part: 0,
                total: 1,
                data: vec![1, 2],
            },
        );
        assert_eq!(state.phase, Phase::Ready);
        let outputs = update(
            &mut state,
            Input::Remote {
                room,
                meta: meta(1),
                op: 9u8,
            },
        );
        assert_eq!(
            outputs,
            vec![Output::Apply {
                meta: meta(1),
                op: 9u8,
            }]
        );
    }

    #[test]
    fn retries_the_same_source_for_the_missing_part() {
        let mut state = State::<u8>::default();
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        update(
            &mut state,
            Input::Joined {
                room: room.clone(),
                members: vec![peer.clone()],
            },
        );
        update(
            &mut state,
            Input::Snapshot {
                room: room.clone(),
                part: 0,
                total: 2,
                data: vec![1],
            },
        );
        let outputs = update(&mut state, Input::Failed);
        assert_eq!(
            outputs,
            vec![Output::Fetch {
                peer,
                room,
                part: 1,
            }]
        );
    }

    #[test]
    fn late_joiner_fetches_then_applies_buffered_ops() {
        let mut joiner = State::<u8>::default();
        let room = identity::RoomName::from("r");
        let peer = identity::PeerId::from("a");
        let outputs = update(
            &mut joiner,
            Input::Joined {
                room: room.clone(),
                members: vec![peer.clone()],
            },
        );
        assert_eq!(
            outputs,
            vec![Output::Fetch {
                peer,
                room: room.clone(),
                part: 0,
            }]
        );
        update(
            &mut joiner,
            Input::Remote {
                room: room.clone(),
                meta: meta(4),
                op: 3u8,
            },
        );
        assert_eq!(joiner.buffered.len(), 1);
        let snapshot = vec![1u8, 2, 3];
        let outputs = update(
            &mut joiner,
            Input::Snapshot {
                room,
                part: 0,
                total: 1,
                data: snapshot.clone(),
            },
        );
        assert_eq!(
            outputs,
            vec![
                Output::Merge { bytes: snapshot },
                Output::Apply {
                    meta: meta(4),
                    op: 3u8,
                },
            ]
        );
        assert_eq!(joiner.phase, Phase::Ready);
    }

    #[test]
    fn buffers_ops_until_the_snapshot_arrives() {
        let mut state = State::<u8>::default();
        let room = identity::RoomName::from("r");
        update(
            &mut state,
            Input::Joined {
                room: room.clone(),
                members: vec![identity::PeerId::from("a")],
            },
        );
        let outputs = update(
            &mut state,
            Input::Remote {
                room: room.clone(),
                meta: meta(1),
                op: 5u8,
            },
        );
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.buffered.len(), 1);
        let outputs = update(
            &mut state,
            Input::Snapshot {
                room,
                part: 0,
                total: 1,
                data: vec![0],
            },
        );
        assert!(matches!(
            outputs.as_slice(),
            [Output::Merge { .. }, Output::Apply { op: 5u8, .. }]
        ));
        assert_eq!(state.buffered, Vec::new());
    }
}
