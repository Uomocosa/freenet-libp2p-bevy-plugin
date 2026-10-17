use serde::{Deserialize, Serialize};

use crate::identity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpMeta {
    pub lamport: u32,
    pub author: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncFrame<O> {
    pub room: identity::RoomName,
    pub meta: OpMeta,
    pub op: O,
}
