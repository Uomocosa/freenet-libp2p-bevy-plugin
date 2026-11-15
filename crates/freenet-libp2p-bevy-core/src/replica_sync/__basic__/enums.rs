use bevy::prelude::SystemSet;
use serde::{Deserialize, Serialize};

use crate::identity;
use crate::replica_sync;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplicaSyncSet {
    Input,
    Drive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplicaSyncTopicMessage<Op> {
    ApplyThisOp {
        room: identity::RoomName,
        meta: replica_sync::OpMeta,
        op: Op,
    },
    CompareSliceContentHashes {
        room: identity::RoomName,
        hashes: replica_sync::ReplicaSliceContentHashes,
    },
}
