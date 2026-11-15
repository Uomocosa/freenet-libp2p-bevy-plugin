use crate::identity;
use crate::p2p;
use crate::peers;
use crate::replica_sync;

pub type ReplicaSliceContentHashes = Vec<(
    replica_sync::ReplicaSliceIndex,
    replica_sync::ReplicaSliceContentHash,
)>;

pub type ZoneDetails<'a> = (
    &'a identity::ZoneName,
    Option<&'a p2p::HistoryKeyPrefix>,
    Option<&'a replica_sync::state_machine::ZoneLiveness>,
    Option<&'a peers::ZoneMembers>,
);
