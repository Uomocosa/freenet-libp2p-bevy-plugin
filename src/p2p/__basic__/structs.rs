use serde::{Deserialize, Serialize};

use crate::identity;
use crate::p2p;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryChunkId {
    pub room: identity::RoomName,
    pub chunk: p2p::ChunkIndex,
}
