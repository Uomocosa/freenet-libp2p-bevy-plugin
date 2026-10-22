use std::time::Duration;

use crate::p2p;
use crate::room_sync;

pub fn merge_and_write_history_records<Replica: room_sync::RoomState>(
    replica: &mut Replica,
    prefix: &p2p::HistoryKeyPrefix,
    stored_manifest: Option<&p2p::HistoryManifest>,
    stored_snapshot: Option<&[u8]>,
    lamport_max: u64,
    per_room_bytes: p2p::HistoryByteCount,
    ttl: Duration,
) -> Vec<p2p::NetCommand> {
    if let Some(bytes) = stored_snapshot
        && let Ok(snapshot) = bincode::deserialize::<Replica::Snapshot>(bytes)
    {
        replica.merge(snapshot);
    }
    let Ok(bytes) = bincode::serialize(&replica.snapshot()) else {
        return Vec::new();
    };
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > *per_room_bytes {
        tracing::error!(
            target: "room_sync",
            bytes = bytes.len(),
            limit = *per_room_bytes,
            "room snapshot exceeds per-room history byte limit; refusing to store"
        );
        return Vec::new();
    }
    let Ok(chunked) = p2p::ChunkedSnapshot::split(&bytes, lamport_max) else {
        return Vec::new();
    };
    let compressed_bytes: u64 = chunked
        .chunks
        .iter()
        .map(|chunk| u64::try_from(chunk.len()).unwrap_or(u64::MAX))
        .fold(0u64, u64::saturating_add);
    if compressed_bytes > *per_room_bytes {
        tracing::error!(
            target: "room_sync",
            bytes = compressed_bytes,
            limit = *per_room_bytes,
            "compressed room snapshot exceeds per-room history byte limit; refusing to store"
        );
        return Vec::new();
    }
    if stored_manifest
        .is_some_and(|manifest| manifest.content_hash == chunked.manifest.content_hash)
    {
        return Vec::new();
    }
    tracing::info!(
        target: "room_sync",
        snapshot_bytes = bytes.len(),
        record_bytes = compressed_bytes,
        chunks = chunked.chunks.len(),
        "room history written"
    );
    let Ok(manifest_bytes) = bincode::serialize(&chunked.manifest) else {
        return Vec::new();
    };
    let mut commands = Vec::new();
    for (index, chunk) in chunked.chunks.into_iter().enumerate() {
        let Ok(chunk_index) = u64::try_from(index) else {
            return Vec::new();
        };
        commands.push(p2p::NetCommand::PutHistoryRecord {
            id: p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
                prefix: prefix.clone(),
                content_hash: chunked.manifest.content_hash.clone(),
                chunk: p2p::ChunkIndex(chunk_index),
            }),
            value: chunk,
            ttl,
        });
    }
    commands.push(p2p::NetCommand::PutHistoryRecord {
        id: p2p::RoomHistoryRecordKind::Manifest(prefix.clone()),
        value: manifest_bytes,
        ttl,
    });
    commands
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use bevy::prelude::*;
    use derive_more::{Deref, DerefMut};
    use serde::{Deserialize, Serialize};

    use super::merge_and_write_history_records;
    use crate::p2p;
    use crate::room_sync;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    struct Op {
        key: u8,
        value: u32,
    }

    #[derive(Resource, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
    struct Map {
        entries: BTreeMap<u8, u32>,
    }

    impl room_sync::RoomState for Map {
        type Op = Op;
        type Snapshot = Vec<(u8, u32)>;

        fn apply(&mut self, _meta: room_sync::OpMeta, op: &Self::Op) {
            self.entries.insert(op.key, op.value);
        }

        fn snapshot(&self) -> Self::Snapshot {
            self.entries.iter().map(|(k, v)| (*k, *v)).collect()
        }

        fn merge(&mut self, snapshot: Self::Snapshot) {
            for (k, v) in snapshot {
                self.entries.insert(k, v);
            }
        }

        fn slice_content_hashes(&self) -> room_sync::ReplicaSliceContentHashes {
            Vec::new()
        }

        fn slice_snapshot(&self, _index: room_sync::ReplicaSliceIndex) -> Self::Snapshot {
            self.snapshot()
        }
    }

    #[test]
    fn test_usage() {
        let mut replica = Map::default();
        replica.entries.insert(1, 10);
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
        assert_eq!(replica.entries.get(&2), Some(&20));
        assert_eq!(commands.len(), 2);
        assert!(matches!(
            commands.last(),
            Some(p2p::NetCommand::PutHistoryRecord {
                id: p2p::RoomHistoryRecordKind::Manifest(_),
                ..
            })
        ));
    }

    #[test]
    fn refuses_snapshot_exceeding_per_room_bytes() {
        let mut replica = Map::default();
        for key in 0..50u8 {
            replica.entries.insert(key, u32::from(key));
        }
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
