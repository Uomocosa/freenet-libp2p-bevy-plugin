use serde::{Deserialize, Serialize};

use crate::net_id;
use crate::p2p;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryChunkId {
    pub room: net_id::RoomName,
    pub chunk: p2p::ChunkIndex,
}
