use crate::p2p;
use crate::peers;
use crate::room_sync;

pub type ReplicaSliceContentHashes = Vec<(
    room_sync::ReplicaSliceIndex,
    room_sync::ReplicaSliceContentHash,
)>;

pub type RoomDetails<'a> = (
    &'a peers::Room,
    Option<&'a p2p::HistoryKeyPrefix>,
    Option<&'a room_sync::state_machine::RoomLiveness>,
    Option<&'a peers::RoomMembers>,
);
