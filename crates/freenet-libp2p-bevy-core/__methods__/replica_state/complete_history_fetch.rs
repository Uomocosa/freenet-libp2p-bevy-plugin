use crate::p2p;
use crate::room_sync;

pub fn complete_history_fetch<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    prefix: p2p::HistoryKeyPrefix,
    manifest: p2p::HistoryManifest,
    bytes: Vec<u8>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    match state.phase {
        room_sync::state_machine::Phase::FetchingHistoryFromKademlia => {
            state.phase = room_sync::state_machine::Phase::Ready;
            let mut outputs = vec![room_sync::state_machine::Output::Merge { bytes }];
            for (meta, op) in state.buffered_ops.drain(..) {
                outputs.push(room_sync::state_machine::Output::Apply { meta, op });
            }
            outputs
        }
        room_sync::state_machine::Phase::Ready => {
            vec![room_sync::state_machine::Output::MergeAndWriteHistory {
                prefix,
                stored_manifest: Some(manifest),
                stored_snapshot: Some(bytes),
            }]
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::complete_history_fetch;
    use crate::p2p;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let manifest = p2p::HistoryManifest {
            lamport_max: 0,
            chunk_count: 0,
            content_hash: p2p::ContentHash::from("hash"),
        };
        let outputs = complete_history_fetch(
            &mut state,
            p2p::HistoryKeyPrefix::from("prefix"),
            manifest,
            Vec::new(),
        );
        assert_eq!(outputs, Vec::new());
    }
}
