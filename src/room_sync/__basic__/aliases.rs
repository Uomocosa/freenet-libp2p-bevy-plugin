use crate::room_sync;

pub type ReplicaSliceContentHashes = Vec<(
    room_sync::ReplicaSliceIndex,
    room_sync::ReplicaSliceContentHash,
)>;
