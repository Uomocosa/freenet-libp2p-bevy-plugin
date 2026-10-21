use crate::p2p;
use crate::room_sync;

pub fn handle_history_record_found<Op>(
    state: &mut room_sync::state_machine::State<Op>,
    id: &p2p::RoomHistoryRecordKind,
    value: Vec<u8>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    match id {
        p2p::RoomHistoryRecordKind::Manifest(prefix) => {
            let Some(pending) = state.kademlia_history.pending_fetch.as_mut() else {
                return Vec::new();
            };
            if &pending.prefix != prefix || pending.manifest.is_some() {
                return Vec::new();
            }
            let Ok(manifest) = bincode::deserialize::<p2p::HistoryManifest>(&value) else {
                return room_sync::state_machine::handle_history_record_not_found(state, id);
            };
            if manifest.chunk_count == 0 {
                return room_sync::state_machine::handle_history_record_not_found(state, id);
            }
            let outputs: Vec<_> = (0..manifest.chunk_count)
                .map(|index| room_sync::state_machine::Output::GetHistoryRecord {
                    id: p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
                        prefix: prefix.clone(),
                        content_hash: manifest.content_hash.clone(),
                        chunk: p2p::ChunkIndex::from(index),
                    }),
                })
                .collect();
            pending.manifest = Some(manifest);
            pending.received_chunks.clear();
            outputs
        }
        p2p::RoomHistoryRecordKind::Chunk(chunk_id) => {
            let Some(pending) = state.kademlia_history.pending_fetch.as_mut() else {
                return Vec::new();
            };
            let Some(manifest) = pending.manifest.clone() else {
                return Vec::new();
            };
            if pending.prefix != chunk_id.prefix
                || manifest.content_hash != chunk_id.content_hash
                || *chunk_id.chunk >= u64::from(manifest.chunk_count)
            {
                return Vec::new();
            }
            pending.received_chunks.insert(chunk_id.chunk, value);
            if u32::try_from(pending.received_chunks.len()).unwrap_or(0) < manifest.chunk_count {
                return Vec::new();
            }
            let prefix = pending.prefix.clone();
            let ordered_chunks: Vec<Vec<u8>> = pending.received_chunks.values().cloned().collect();
            let Ok(bytes) = p2p::join_chunks_into_snapshot(&ordered_chunks, &manifest) else {
                return room_sync::state_machine::handle_history_record_not_found(state, id);
            };
            state.kademlia_history.pending_fetch = None;
            complete_history_fetch(state, prefix, manifest, bytes)
        }
    }
}

// needed helper: emits the phase-appropriate output once all chunks of a Kademlia snapshot are verified
fn complete_history_fetch<Op>(
    state: &mut room_sync::state_machine::State<Op>,
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
    use std::collections::BTreeMap;

    use super::handle_history_record_found;
    use crate::identity;
    use crate::p2p;
    use crate::room_sync::state_machine::{Output, PendingHistoryFetch, Phase, State};

    #[test]
    fn test_usage() {
        let mut state = State::<u8>::default();
        let prefix = p2p::HistoryKeyPrefix::from("abc");
        state.room = Some(identity::RoomName::from("r"));
        state.phase = Phase::FetchingHistoryFromKademlia;
        state.kademlia_history.is_enabled = true;
        state.kademlia_history.prefix = Some(prefix.clone());
        state.kademlia_history.pending_fetch = Some(PendingHistoryFetch {
            prefix: prefix.clone(),
            manifest: None,
            received_chunks: BTreeMap::new(),
        });
        let split = p2p::split_snapshot_into_chunks(&[1, 2, 3], 4).expect("split");
        let manifest_bytes = bincode::serialize(&split.manifest).expect("manifest");
        let manifest_out = handle_history_record_found(
            &mut state,
            &p2p::RoomHistoryRecordKind::Manifest(prefix.clone()),
            manifest_bytes,
        );
        let chunk_kind = p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
            prefix,
            content_hash: split.manifest.content_hash,
            chunk: p2p::ChunkIndex(0),
        });
        assert_eq!(
            manifest_out,
            vec![Output::GetHistoryRecord {
                id: chunk_kind.clone(),
            }]
        );
        let chunk_out =
            handle_history_record_found(&mut state, &chunk_kind, split.chunks[0].clone());
        assert_eq!(state.phase, Phase::Ready);
        assert_eq!(
            chunk_out,
            vec![Output::Merge {
                bytes: vec![1, 2, 3]
            }]
        );
    }
}
