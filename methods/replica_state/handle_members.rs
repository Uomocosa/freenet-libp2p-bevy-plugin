use crate::identity;
use crate::p2p;
use crate::room_sync;

pub fn handle_members<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    members: Vec<identity::PeerId>,
    prefix: p2p::HistoryKeyPrefix,
    lobby_status: room_sync::state_machine::LobbyRoomStatus,
) -> Vec<room_sync::state_machine::Output<Op>> {
    let mut members = members;
    members.sort();
    state.member_snapshot.members = members;
    state.kademlia_history.prefix = Some(prefix);
    state.kademlia_history.lobby_status = lobby_status;
    match state.phase {
        room_sync::state_machine::Phase::FetchingSnapshotFromRoomMember => state.fetch_next(),
        room_sync::state_machine::Phase::FetchingHistoryFromKademlia => {
            if !lobby_status.is_room_alive_in_lobby {
                let Some(room) = state.room.clone() else {
                    return Vec::new();
                };
                state.phase = room_sync::state_machine::Phase::Failed;
                state.kademlia_history.pending_fetch = None;
                return vec![room_sync::state_machine::Output::RoomGone { room }];
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::handle_members;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync::state_machine::{LobbyRoomStatus, ReplicaState};

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let outputs = handle_members(
            &mut state,
            vec![identity::PeerId::from("peer")],
            p2p::HistoryKeyPrefix::from("prefix"),
            LobbyRoomStatus::default(),
        );
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.member_snapshot.members.len(), 1);
    }
}
