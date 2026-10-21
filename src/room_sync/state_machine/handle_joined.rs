use crate::identity;
use crate::p2p;
use crate::room_sync;

pub fn handle_joined<Op>(
    state: &mut room_sync::state_machine::State<Op>,
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
    let next = room_sync::state_machine::fetch_next(state);
    if !next.is_empty() {
        return next;
    }
    if state.kademlia_history.is_enabled {
        return room_sync::state_machine::start_kademlia_history_fetch(state);
    }
    Vec::new()
}

// no test_usage necessary
