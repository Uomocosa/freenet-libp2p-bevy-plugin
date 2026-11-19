use std::collections::BTreeMap;

use crate::p2p;
use crate::spectator;

pub fn handle_history_record<Snap>(
    bootstrap: &mut spectator::HistoryBootstrap<Snap>,
    id: &p2p::ZoneHistoryRecordKind,
    value: Vec<u8>,
) -> spectator::BootstrapOutcome {
    match id {
        p2p::ZoneHistoryRecordKind::Manifest(prefix) => {
            let Ok(manifest) = bincode::deserialize::<p2p::HistoryManifest>(&value) else {
                return spectator::BootstrapOutcome::Ignored;
            };
            if manifest.chunk_count == 0 {
                return spectator::BootstrapOutcome::Ignored;
            }
            let requests = (0..manifest.chunk_count)
                .map(|index| {
                    p2p::ZoneHistoryRecordKind::Chunk(p2p::HistoryChunkId {
                        prefix: prefix.clone(),
                        content_hash: manifest.content_hash.clone(),
                        chunk: p2p::ChunkIndex::from(index),
                    })
                })
                .collect();
            bootstrap.pending.insert(
                prefix.clone(),
                spectator::PendingHistory {
                    manifest: Some(manifest),
                    chunks: BTreeMap::default(),
                },
            );
            spectator::BootstrapOutcome::RequestChunks(requests)
        }
        p2p::ZoneHistoryRecordKind::Chunk(chunk_id) => {
            let Some(pending) = bootstrap.pending.get_mut(&chunk_id.prefix) else {
                return spectator::BootstrapOutcome::Ignored;
            };
            let Some(manifest) = pending.manifest.clone() else {
                return spectator::BootstrapOutcome::Ignored;
            };
            if manifest.content_hash != chunk_id.content_hash
                || *chunk_id.chunk >= u64::from(manifest.chunk_count)
            {
                return spectator::BootstrapOutcome::Ignored;
            }
            pending.chunks.insert(chunk_id.chunk, value);
            if u32::try_from(pending.chunks.len()).unwrap_or(0) < manifest.chunk_count {
                return spectator::BootstrapOutcome::Ignored;
            }
            let ordered: Vec<Vec<u8>> = pending.chunks.values().cloned().collect();
            let Ok(snapshot) = p2p::ChunkedSnapshot::join(&ordered, &manifest) else {
                return spectator::BootstrapOutcome::Ignored;
            };
            let history_prefix = chunk_id.prefix.clone();
            bootstrap.pending.remove(&history_prefix);
            spectator::BootstrapOutcome::Snapshot {
                history_prefix,
                snapshot,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::handle_history_record;
    use crate::p2p;
    use crate::spectator;

    #[test]
    fn test_usage() {
        let mut bootstrap = spectator::HistoryBootstrap::<Vec<u8>>::default();
        let payload = vec![7_u8, 8, 9];
        let prefix = p2p::HistoryKeyPrefix::from("prefix");
        let split = p2p::ChunkedSnapshot::try_from_payload(&payload, 1).expect("split");
        let manifest = split.manifest;
        let chunks = split.chunks;
        let expected = payload;
        let manifest_bytes = bincode::serialize(&manifest).expect("manifest");
        assert!(matches!(
            handle_history_record(
                &mut bootstrap,
                &p2p::ZoneHistoryRecordKind::Manifest(prefix.clone()),
                manifest_bytes,
            ),
            spectator::BootstrapOutcome::RequestChunks(_)
        ));
        let outcome = handle_history_record(
            &mut bootstrap,
            &p2p::ZoneHistoryRecordKind::Chunk(p2p::HistoryChunkId {
                prefix: prefix.clone(),
                content_hash: manifest.content_hash,
                chunk: p2p::ChunkIndex(0),
            }),
            chunks[0].clone(),
        );
        assert_eq!(
            outcome,
            spectator::BootstrapOutcome::Snapshot {
                history_prefix: prefix,
                snapshot: expected,
            }
        );
        assert!(bootstrap.pending.is_empty());
    }

    #[test]
    fn two_zones_bootstrap_side_by_side() {
        let mut bootstrap = spectator::HistoryBootstrap::<Vec<u8>>::default();
        let first_payload = vec![1_u8];
        let first_prefix = p2p::HistoryKeyPrefix::from("first");
        let first_split = p2p::ChunkedSnapshot::try_from_payload(&first_payload, 1).expect("split");
        let first_manifest = first_split.manifest;
        let first_chunks = first_split.chunks;
        let second_prefix = p2p::HistoryKeyPrefix::from("second");
        let second_split = p2p::ChunkedSnapshot::try_from_payload(&[2_u8], 1).expect("split");
        let second_manifest = second_split.manifest;
        let second_chunks = second_split.chunks;
        for (prefix, manifest) in [
            (first_prefix.clone(), &first_manifest),
            (second_prefix.clone(), &second_manifest),
        ] {
            handle_history_record(
                &mut bootstrap,
                &p2p::ZoneHistoryRecordKind::Manifest(prefix),
                bincode::serialize(manifest).expect("manifest"),
            );
        }
        handle_history_record(
            &mut bootstrap,
            &p2p::ZoneHistoryRecordKind::Chunk(p2p::HistoryChunkId {
                prefix: second_prefix,
                content_hash: second_manifest.content_hash,
                chunk: p2p::ChunkIndex(0),
            }),
            second_chunks[0].clone(),
        );
        assert!(bootstrap.pending.contains_key(&first_prefix));
        assert_eq!(
            handle_history_record(
                &mut bootstrap,
                &p2p::ZoneHistoryRecordKind::Chunk(p2p::HistoryChunkId {
                    prefix: first_prefix.clone(),
                    content_hash: first_manifest.content_hash,
                    chunk: p2p::ChunkIndex(0),
                }),
                first_chunks[0].clone(),
            ),
            spectator::BootstrapOutcome::Snapshot {
                history_prefix: first_prefix,
                snapshot: first_payload,
            }
        );
    }
}
