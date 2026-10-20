use serde::{Deserialize, Serialize};

use crate::identity;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkedSnapshot {
    pub manifest: HistoryManifest,
    pub chunks: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerConfig {
    pub mode: p2p::TransportMode,
    pub mdns: p2p::MdnsMode,
    pub game: identity::GameId,
    pub gossipsub: p2p::GossipsubConfig,
}
