use serde::{Deserialize, Serialize};

use crate::p2p;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryChunkId {
    pub prefix: p2p::HistoryKeyPrefix,
    pub content_hash: p2p::ContentHash,
    pub chunk: p2p::ChunkIndex,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryManifest {
    pub lamport_max: u64,
    pub chunk_count: u32,
    pub content_hash: p2p::ContentHash,
}
