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
            bootstrap.prefix = Some(prefix.clone());
            bootstrap.chunks.clear();
            let requests = (0..manifest.chunk_count)
                .map(|index| {
                    p2p::ZoneHistoryRecordKind::Chunk(p2p::HistoryChunkId {
                        prefix: prefix.clone(),
                        content_hash: manifest.content_hash.clone(),
                        chunk: p2p::ChunkIndex::from(index),
                    })
                })
                .collect();
            bootstrap.manifest = Some(manifest);
            spectator::BootstrapOutcome::RequestChunks(requests)
        }
        p2p::ZoneHistoryRecordKind::Chunk(chunk_id) => {
            let Some(manifest) = bootstrap.manifest.clone() else {
                return spectator::BootstrapOutcome::Ignored;
            };
            if bootstrap.prefix.as_ref() != Some(&chunk_id.prefix)
                || manifest.content_hash != chunk_id.content_hash
                || *chunk_id.chunk >= u64::from(manifest.chunk_count)
            {
                return spectator::BootstrapOutcome::Ignored;
            }
            bootstrap.chunks.insert(chunk_id.chunk, value);
            if u32::try_from(bootstrap.chunks.len()).unwrap_or(0) < manifest.chunk_count {
                return spectator::BootstrapOutcome::Ignored;
            }
            let ordered: Vec<Vec<u8>> = bootstrap.chunks.values().cloned().collect();
            let Ok(bytes) = p2p::ChunkedSnapshot::join(&ordered, &manifest) else {
                return spectator::BootstrapOutcome::Ignored;
            };
            bootstrap.reset();
            spectator::BootstrapOutcome::Snapshot(bytes)
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
        let prefix = p2p::HistoryKeyPrefix::from("prefix");
        let payload = vec![7_u8, 8, 9];
        let split = p2p::ChunkedSnapshot::try_from_payload(&payload, 1).expect("split");
        let manifest = bincode::serialize(&split.manifest).expect("manifest");
        let requests = handle_history_record(
            &mut bootstrap,
            &p2p::ZoneHistoryRecordKind::Manifest(prefix.clone()),
            manifest,
        );
        assert!(matches!(
            requests,
            spectator::BootstrapOutcome::RequestChunks(_)
        ));
        let chunk = handle_history_record(
            &mut bootstrap,
            &p2p::ZoneHistoryRecordKind::Chunk(p2p::HistoryChunkId {
                prefix,
                content_hash: split.manifest.content_hash,
                chunk: p2p::ChunkIndex(0),
            }),
            split.chunks[0].clone(),
        );
        assert_eq!(chunk, spectator::BootstrapOutcome::Snapshot(payload));
        assert!(bootstrap.manifest.is_none());
    }
}
