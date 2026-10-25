use crate::p2p;
use crate::room_sync;

pub fn handle_history_record_not_found<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    id: &p2p::RoomHistoryRecordKind,
) -> Vec<room_sync::state_machine::Output<Op>> {
    let prefix = id.record_prefix().clone();
    if state
        .kademlia_history
        .pending_fetch
        .as_ref()
        .map(|pending| &pending.prefix)
        != Some(&prefix)
    {
        return Vec::new();
    }
    state.kademlia_history.pending_fetch = None;
    match state.phase {
        room_sync::state_machine::Phase::Ready => {
            vec![room_sync::state_machine::Output::MergeAndWriteHistory {
                prefix,
                stored_manifest: None,
                stored_snapshot: None,
            }]
        }
        room_sync::state_machine::Phase::FetchingHistoryFromKademlia => {
            let Some(room) = state.room.clone() else {
                return Vec::new();
            };
            let lobby_status = state.kademlia_history.lobby_status;
            if !lobby_status.is_room_alive_in_lobby {
                state.phase = room_sync::state_machine::Phase::Failed;
                return vec![room_sync::state_machine::Output::RoomGone { room }];
            }
            if !lobby_status.has_other_lobby_holders && state.kademlia_history.fetch_retries == 0 {
                state.phase = room_sync::state_machine::Phase::Ready;
                let mut outputs = Vec::new();
                for (meta, op) in state.buffered_ops.drain(..) {
                    outputs.push(room_sync::state_machine::Output::Apply { meta, op });
                }
                return outputs;
            }
            if state.kademlia_history.fetch_retries < state.kademlia_history.max_fetch_retries {
                state.kademlia_history.fetch_retries =
                    state.kademlia_history.fetch_retries.saturating_add(1);
                return state.start_kademlia_history_fetch();
            }
            state.phase = room_sync::state_machine::Phase::Failed;
            vec![room_sync::state_machine::Output::HistoryUnavailable { room }]
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::handle_history_record_not_found;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync::state_machine::{
        LobbyRoomStatus, Output, PendingHistoryFetch, Phase, ReplicaState,
    };

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let room = identity::RoomName::from("r");
        let prefix = p2p::HistoryKeyPrefix::from("abc");
        state.room = Some(room.clone());
        state.phase = Phase::FetchingHistoryFromKademlia;
        state.kademlia_history.is_enabled = true;
        state.kademlia_history.prefix = Some(prefix.clone());
        state.kademlia_history.lobby_status = LobbyRoomStatus {
            is_room_alive_in_lobby: false,
            has_other_lobby_holders: false,
        };
        state.kademlia_history.pending_fetch = Some(PendingHistoryFetch {
            prefix: prefix.clone(),
            manifest: None,
            received_chunks: BTreeMap::new(),
        });
        let outputs = handle_history_record_not_found(
            &mut state,
            &p2p::RoomHistoryRecordKind::Manifest(prefix),
        );
        assert_eq!(state.phase, Phase::Failed);
        assert_eq!(outputs, vec![Output::RoomGone { room }]);
    }
}
