use crate::identity;
use crate::p2p;
use crate::room_sync;

pub fn handle_joined<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    room: identity::RoomName,
    prefix: p2p::HistoryKeyPrefix,
    members: Vec<identity::PeerId>,
    lobby_status: room_sync::state_machine::LobbyRoomStatus,
) -> Vec<room_sync::state_machine::Output<Op>> {
    state.room = Some(room);
    state.phase = room_sync::state_machine::Phase::FetchingSnapshotFromRoomMember;
    state.buffered_ops.clear();
    let mut members = members;
    members.sort();
    state.member_snapshot = room_sync::state_machine::MemberSnapshotFetchState {
        total_parts: 0,
        received_parts: std::collections::BTreeMap::new(),
        members,
        asked_members: Vec::new(),
        current_source: None,
    };
    state.slice_anti_entropy.fetches_awaiting_reply.clear();
    state.kademlia_history.prefix = Some(prefix);
    state.kademlia_history.lobby_status = lobby_status;
    state.kademlia_history.fetch_retries = 0;
    state.kademlia_history.pending_fetch = None;
    let next = state.fetch_next();
    if !next.is_empty() {
        return next;
    }
    if state.kademlia_history.is_enabled {
        return state.start_kademlia_history_fetch();
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::handle_joined;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync;
    use crate::room_sync::state_machine::{LobbyRoomStatus, ReplicaState};

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let outputs = handle_joined(
            &mut state,
            identity::RoomName::from("room"),
            p2p::HistoryKeyPrefix::from("prefix"),
            vec![identity::PeerId::from("peer")],
            LobbyRoomStatus::default(),
        );
        assert_eq!(outputs.len(), 1);
        assert_eq!(
            state.phase,
            room_sync::state_machine::Phase::FetchingSnapshotFromRoomMember
        );
    }
}
