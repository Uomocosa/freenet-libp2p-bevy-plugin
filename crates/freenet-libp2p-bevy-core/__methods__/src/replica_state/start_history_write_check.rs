use std::collections::BTreeMap;

use crate::p2p;
use crate::room_sync;

pub fn start_history_write_check<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.phase != room_sync::state_machine::Phase::Ready
        || !state.kademlia_history.is_enabled
        || state.kademlia_history.pending_fetch.is_some()
    {
        return Vec::new();
    }
    let Some(prefix) = state.kademlia_history.prefix.clone() else {
        return Vec::new();
    };
    state.kademlia_history.pending_fetch = Some(room_sync::state_machine::PendingHistoryFetch {
        prefix: prefix.clone(),
        manifest: None,
        received_chunks: BTreeMap::new(),
        records_requested_at: None,
    });
    vec![room_sync::state_machine::Output::GetHistoryRecord {
        id: p2p::RoomHistoryRecordKind::Manifest(prefix),
    }]
}

#[cfg(test)]
mod tests {
    use super::start_history_write_check;
    use crate::p2p;
    use crate::room_sync::state_machine::{Output, Phase, ReplicaState};

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let prefix = p2p::HistoryKeyPrefix::from("abc");
        state.phase = Phase::Ready;
        state.kademlia_history.is_enabled = true;
        state.kademlia_history.prefix = Some(prefix.clone());
        let outputs = start_history_write_check(&mut state);
        assert_eq!(
            outputs,
            vec![Output::GetHistoryRecord {
                id: p2p::RoomHistoryRecordKind::Manifest(prefix),
            }]
        );
        assert_eq!(start_history_write_check(&mut state), Vec::new());
    }
}
