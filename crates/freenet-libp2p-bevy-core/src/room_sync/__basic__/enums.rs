use bevy::prelude::SystemSet;
use serde::{Deserialize, Serialize};

use crate::identity;
use crate::room_sync;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomSyncSet {
    Input,
    Drive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplicaSyncTopicMessage<Op> {
    ApplyThisOp {
        room: identity::RoomName,
        meta: room_sync::OpMeta,
        op: Op,
    },
    CompareSliceContentHashes {
        room: identity::RoomName,
        hashes: room_sync::ReplicaSliceContentHashes,
    },
}
