use std::time::Duration;

use crate::p2p;
use crate::replica_sync;

pub fn merge_and_write_history_records<Replica: replica_sync::traits::ReplicatedState>(
    replica: &mut Replica,
    prefix: &p2p::HistoryKeyPrefix,
    stored_manifest: Option<&p2p::HistoryManifest>,
    stored_snapshot: Option<&[u8]>,
    lamport_max: u64,
    per_zone_bytes: p2p::HistoryByteCount,
    ttl: Duration,
) -> Vec<p2p::Libp2pCommand> {
    if let Some(bytes) = stored_snapshot
        && let Ok(snapshot) = bincode::deserialize::<Replica::Snapshot>(bytes)
    {
        replica.merge(snapshot);
    }
    let Ok(bytes) = bincode::serialize(&replica.snapshot()) else {
        return Vec::new();
    };
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > *per_zone_bytes {
        tracing::error!(
            target: "replica_sync",
            bytes = bytes.len(),
            limit = *per_zone_bytes,
            "zone snapshot exceeds per-zone history byte limit; refusing to store"
        );
        return Vec::new();
    }
    let Ok(chunked) = p2p::ChunkedSnapshot::try_from_payload(&bytes, lamport_max) else {
        return Vec::new();
    };
    let compressed_bytes: u64 = chunked
        .chunks
        .iter()
        .map(|chunk| u64::try_from(chunk.len()).unwrap_or(u64::MAX))
        .fold(0u64, u64::saturating_add);
    if compressed_bytes > *per_zone_bytes {
        tracing::error!(
            target: "replica_sync",
            bytes = compressed_bytes,
            limit = *per_zone_bytes,
            "compressed zone snapshot exceeds per-zone history byte limit; refusing to store"
        );
        return Vec::new();
    }
    if stored_manifest
        .is_some_and(|manifest| manifest.content_hash == chunked.manifest.content_hash)
    {
        return Vec::new();
    }
    tracing::info!(
        target: "replica_sync",
        snapshot_bytes = bytes.len(),
        record_bytes = compressed_bytes,
        chunks = chunked.chunks.len(),
        "zone history written"
    );
    let Ok(manifest_bytes) = bincode::serialize(&chunked.manifest) else {
        return Vec::new();
    };
    let mut commands = Vec::new();
    for (index, chunk) in chunked.chunks.into_iter().enumerate() {
        let Ok(chunk_index) = u64::try_from(index) else {
            return Vec::new();
        };
        commands.push(p2p::Libp2pCommand::PutHistoryRecord {
            id: p2p::ZoneHistoryRecordKind::Chunk(p2p::HistoryChunkId {
                prefix: prefix.clone(),
                content_hash: chunked.manifest.content_hash.clone(),
                chunk: p2p::ChunkIndex(chunk_index),
            }),
            value: chunk,
            ttl,
        });
    }
    commands.push(p2p::Libp2pCommand::PutHistoryRecord {
        id: p2p::ZoneHistoryRecordKind::Manifest(prefix.clone()),
        value: manifest_bytes,
        ttl,
    });
    commands
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::merge_and_write_history_records;
    use crate::p2p;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut replica = testing::KeyValueReplica::default();
        replica.stored_snapshot.push((1, 10));
        let stored = bincode::serialize(&vec![(2u8, 20u32)]).expect("serialize");
        let prefix = p2p::HistoryKeyPrefix::from("prefix");
        let commands = merge_and_write_history_records(
            &mut replica,
            &prefix,
            None,
            Some(&stored),
            5,
            p2p::HistoryByteCount(64 * 1024 * 1024),
            Duration::from_secs(90),
        );
        assert!(replica.stored_snapshot.contains(&(2, 20)));
        assert_eq!(commands.len(), 2);
        assert!(matches!(
            commands.last(),
            Some(p2p::Libp2pCommand::PutHistoryRecord {
                id: p2p::ZoneHistoryRecordKind::Manifest(_),
                ..
            })
        ));
    }

    #[test]
    fn refuses_snapshot_exceeding_per_zone_bytes() {
        let mut replica = testing::KeyValueReplica {
            stored_snapshot: (0..50u8).map(|key| (key, u32::from(key))).collect(),
            ..Default::default()
        };
        let prefix = p2p::HistoryKeyPrefix::from("prefix");
        let commands = merge_and_write_history_records(
            &mut replica,
            &prefix,
            None,
            None,
            1,
            p2p::HistoryByteCount(4),
            Duration::from_secs(90),
        );
        assert_eq!(commands, Vec::new());
    }
}
