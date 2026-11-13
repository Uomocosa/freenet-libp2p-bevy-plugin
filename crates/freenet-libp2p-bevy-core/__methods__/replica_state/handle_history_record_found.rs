use crate::p2p;
use crate::room_sync;

pub fn handle_history_record_found<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
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
                return state.handle_history_record_not_found(id);
            };
            if manifest.chunk_count == 0 {
                return state.handle_history_record_not_found(id);
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
            let Ok(bytes) = p2p::ChunkedSnapshot::join(&ordered_chunks, &manifest) else {
                return state.handle_history_record_not_found(id);
            };
            state.kademlia_history.pending_fetch = None;
            state.complete_history_fetch(prefix, manifest, bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::handle_history_record_found;
    use crate::p2p;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let id = p2p::RoomHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("prefix"));
        assert_eq!(
            handle_history_record_found(&mut state, &id, Vec::new()),
            Vec::new()
        );
    }
}
