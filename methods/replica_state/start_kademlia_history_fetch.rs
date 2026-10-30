use std::collections::BTreeMap;

use crate::p2p;
use crate::room_sync;

pub fn start_kademlia_history_fetch<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    let Some(prefix) = state.kademlia_history.prefix.clone() else {
        return Vec::new();
    };
    state.phase = room_sync::state_machine::Phase::FetchingHistoryFromKademlia;
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
    use super::start_kademlia_history_fetch;
    use crate::p2p;
    use crate::room_sync::state_machine::{Output, Phase, ReplicaState};

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let prefix = p2p::HistoryKeyPrefix::from("abc");
        state.kademlia_history.prefix = Some(prefix.clone());
        let outputs = start_kademlia_history_fetch(&mut state);
        assert_eq!(state.phase, Phase::FetchingHistoryFromKademlia);
        assert_eq!(
            outputs,
            vec![Output::GetHistoryRecord {
                id: p2p::RoomHistoryRecordKind::Manifest(prefix),
            }]
        );
    }
}
