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
        zone: identity::ZoneName,
        meta: replica_sync::OpMeta,
        op: Op,
    },
    CompareSliceContentHashes {
        zone: identity::ZoneName,
        hashes: replica_sync::ReplicaSliceContentHashes,
    },
}
