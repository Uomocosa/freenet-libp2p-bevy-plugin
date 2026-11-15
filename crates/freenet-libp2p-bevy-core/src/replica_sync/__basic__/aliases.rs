use crate::identity;
use crate::p2p;
use crate::peers;
use crate::replica_sync;

pub type ReplicaSliceContentHashes = Vec<(
    replica_sync::ReplicaSliceIndex,
    replica_sync::ReplicaSliceContentHash,
)>;

pub type RoomDetails<'a> = (
    &'a identity::RoomName,
    Option<&'a p2p::HistoryKeyPrefix>,
    Option<&'a replica_sync::state_machine::RoomLiveness>,
    Option<&'a peers::RoomMembers>,
);
