#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::aliases::ReplicaSliceContentHashes;
pub use basic::constants::PART_BYTES;
pub use basic::enums::ReplicaSyncTopicMessage;
pub use basic::resources::{AntiEntropyTimer, HistoryWriteTimer, SyncGame};
pub use basic::structs::OpMeta;

mod anti_entropy_config;
pub use anti_entropy_config::AntiEntropyConfig;

mod history_config;
pub use history_config::HistoryConfig;

mod join_parts;
pub use join_parts::join_parts;

mod lamport_clock;
pub use lamport_clock::LamportClock;

mod merge_and_write_history_records;
pub use merge_and_write_history_records::merge_and_write_history_records;

mod peer_author;
pub use peer_author::peer_author;

mod replica_slice_content_hash;
pub use replica_slice_content_hash::ReplicaSliceContentHash;

mod replica_slice_index;
pub use replica_slice_index::ReplicaSliceIndex;

mod room_state;
pub use room_state::RoomState;

mod pending_ops;
pub use pending_ops::PendingOps;

mod sync_plugin;
pub use sync_plugin::SyncPlugin;

mod split_parts;
pub use split_parts::split_parts;

pub mod state_machine;

mod sync_topic;
pub use sync_topic::sync_topic;

#[cfg(feature = "discovery")]
pub mod bevy_systems;
