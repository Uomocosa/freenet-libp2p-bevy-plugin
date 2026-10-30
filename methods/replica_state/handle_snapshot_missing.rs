use crate::room_sync;

pub const fn handle_snapshot_missing<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    state.member_snapshot.part_requested_at = None;
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::handle_snapshot_missing;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync;
    use crate::room_sync::state_machine::{Input, LobbyRoomStatus, Output, Phase, ReplicaState};

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        assert_eq!(handle_snapshot_missing(&mut state), Vec::new());
    }

    #[test]
    fn missing_part_moves_the_fetch_to_the_next_member() {
        let mut state = ReplicaState::<u8>::default();
        let room = identity::RoomName::from("r");
        let first = identity::PeerId::from("a");
        let second = identity::PeerId::from("b");
        let now = room_sync::ElapsedSinceAppStart::default();
        state.update(
            Input::Joined {
                room: room.clone(),
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: vec![first, second.clone()],
                lobby_status: LobbyRoomStatus::default(),
            },
            now,
        );
        let outputs = state.update(Input::SnapshotMissingFromRoomMember, now);
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromRoomMember {
                peer: second,
                room,
                part: 0,
            }]
        );
        assert_eq!(state.phase, Phase::FetchingSnapshotFromRoomMember);
    }
}
