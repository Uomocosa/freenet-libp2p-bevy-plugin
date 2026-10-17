use serde::{Deserialize, Serialize};

use crate::identity;
use crate::p2p;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryChunkId {
    pub room: identity::RoomName,
    pub chunk: p2p::ChunkIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerConfig {
    pub mode: p2p::TransportMode,
    pub mdns: p2p::MdnsMode,
    pub game: identity::GameId,
    pub gossipsub: p2p::GossipsubConfig,
}
