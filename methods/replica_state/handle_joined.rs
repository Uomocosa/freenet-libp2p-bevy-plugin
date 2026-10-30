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
        part_requested_at: None,
    };
    state.slice_anti_entropy.fetches_awaiting_reply.clear();
    state.kademlia_history.prefix = Some(prefix);
    state.kademlia_history.lobby_status = lobby_status;
    state.kademlia_history.fetch_retries = 0;
    state.kademlia_history.pending_fetch = None;
    let next = state.fetch_next();
    let is_fetching_snapshot = !next.is_empty();
    tracing::debug!(target: "room_sync", "snapshot joined room={:?} members={:?} is_fetching_snapshot={} history_enabled={}", state.room, state.member_snapshot.members, is_fetching_snapshot, state.kademlia_history.is_enabled);
    if is_fetching_snapshot {
        return next;
    }
    state.finish_member_fetch()
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

    #[test]
    fn a_lone_room_creator_without_history_becomes_ready() {
        let mut state = ReplicaState::<u8>::default();
        let room = identity::RoomName::from("room");
        let outputs = handle_joined(
            &mut state,
            room.clone(),
            p2p::HistoryKeyPrefix::from("prefix"),
            Vec::new(),
            LobbyRoomStatus::default(),
        );
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.phase, room_sync::state_machine::Phase::Ready);
        let meta = room_sync::OpMeta {
            lamport: 1,
            author: 2,
        };
        let outputs = state.handle_remote(&room, meta, 7_u8);
        assert_eq!(
            outputs,
            vec![room_sync::state_machine::Output::Apply { meta, op: 7_u8 }]
        );
    }
}
