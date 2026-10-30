use crate::room_sync;

pub fn update<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    input: room_sync::state_machine::Input<Op>,
    now: room_sync::ElapsedSinceAppStart,
) -> Vec<room_sync::state_machine::Output<Op>> {
    let mut outputs = state.dispatch_input(input);
    let expired = state.expire_overdue_requests(now);
    outputs.extend(expired);
    let settled = state.settle_waiting_phase(&outputs);
    outputs.extend(settled);
    state.record_request_send_times(&outputs, now);
    outputs
}

#[cfg(test)]
mod tests {
    use super::update;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync;
    use crate::room_sync::state_machine::{Input, LobbyRoomStatus, Phase, ReplicaState};

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let now = room_sync::ElapsedSinceAppStart::default();
        let outputs = update(
            &mut state,
            Input::Joined {
                room: identity::RoomName::from("r"),
                prefix: p2p::HistoryKeyPrefix::from("prefix"),
                members: vec![identity::PeerId::from("a")],
                lobby_status: LobbyRoomStatus::default(),
            },
            now,
        );
        assert_eq!(outputs.len(), 1);
        assert_eq!(state.phase, Phase::FetchingSnapshotFromRoomMember);
        assert_eq!(state.member_snapshot.part_requested_at, Some(now));
    }
}
