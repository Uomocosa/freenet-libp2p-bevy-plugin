#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::enums::{Input, Output, Phase};
pub use basic::structs::{
    KademliaHistoryState, LobbyRoomStatus, PendingHistoryFetch, SliceAntiEntropyState,
};

mod member_snapshot_fetch_state;
pub use member_snapshot_fetch_state::MemberSnapshotFetchState;

mod replica_state;
pub use replica_state::ReplicaState;
