use crate::identity;
use crate::p2p;
use crate::room_sync;

pub fn handle_members<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    members: Vec<identity::PeerId>,
    prefix: p2p::HistoryKeyPrefix,
    lobby_status: room_sync::state_machine::RoomLiveness,
) -> Vec<room_sync::state_machine::Output<Op>> {
    let mut members = members;
    members.sort();
    state.member_snapshot.members = members;
    state.kademlia_history.prefix = Some(prefix);
    state.kademlia_history.lobby_status = lobby_status;
    match state.phase {
        room_sync::state_machine::Phase::FetchingSnapshotFromRoomMember => state.fetch_next(),
        room_sync::state_machine::Phase::FetchingHistoryFromKademlia => {
            if !lobby_status.is_room_still_listed {
                let Some(room) = state.room.clone() else {
                    return Vec::new();
                };
                state.phase = room_sync::state_machine::Phase::Failed;
                state.kademlia_history.pending_fetch = None;
                return vec![room_sync::state_machine::Output::RoomGone { room }];
            }
            let outputs = state.fetch_next();
            if !outputs.is_empty() {
                state.phase = room_sync::state_machine::Phase::FetchingSnapshotFromRoomMember;
                state.kademlia_history.pending_fetch = None;
                state.kademlia_history.fetch_retries = 0;
            }
            outputs
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::handle_members;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync::state_machine::{
        Output, PendingHistoryFetch, Phase, ReplicaState, RoomLiveness,
    };

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let outputs = handle_members(
            &mut state,
            vec![identity::PeerId::from("peer")],
            p2p::HistoryKeyPrefix::from("prefix"),
            RoomLiveness::default(),
        );
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.member_snapshot.members.len(), 1);
    }

    #[test]
    fn a_member_connecting_during_the_kademlia_fetch_is_asked_for_the_board() {
        let mut state = ReplicaState::<u8>::default();
        let room = identity::RoomName::from("r");
        let prefix = p2p::HistoryKeyPrefix::from("prefix");
        let painter = identity::PeerId::from("painter");
        state.room = Some(room.clone());
        state.phase = Phase::FetchingHistoryFromKademlia;
        state.kademlia_history.is_enabled = true;
        state.kademlia_history.fetch_retries = 2;
        state.kademlia_history.pending_fetch = Some(PendingHistoryFetch {
            prefix: prefix.clone(),
            manifest: None,
            received_chunks: BTreeMap::new(),
            records_requested_at: None,
        });
        let outputs = handle_members(
            &mut state,
            vec![painter.clone()],
            prefix,
            RoomLiveness {
                is_room_still_listed: true,
                has_other_history_holders: true,
            },
        );
        assert_eq!(
            outputs,
            vec![Output::FetchSnapshotPartFromRoomMember {
                peer: painter,
                room,
                part: 0,
            }]
        );
        assert_eq!(state.phase, Phase::FetchingSnapshotFromRoomMember);
        assert_eq!(state.kademlia_history.pending_fetch, None);
        assert_eq!(state.kademlia_history.fetch_retries, 0);
    }
}
