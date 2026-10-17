#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::constants::PART_BYTES;
pub use basic::resources::RoomSyncGame;
pub use basic::structs::{OpMeta, SyncFrame};

mod join_parts;
pub use join_parts::join_parts;

mod lamport_clock;
pub use lamport_clock::LamportClock;

mod peer_author;
pub use peer_author::peer_author;

mod room_state;
pub use room_state::RoomState;

mod room_sync;
pub use room_sync::RoomSync;

mod sync_plugin;
pub use sync_plugin::SyncPlugin;

mod split_parts;
pub use split_parts::split_parts;

pub mod state_machine;

mod sync_topic;
pub use sync_topic::sync_topic;

#[cfg(feature = "discovery")]
pub mod bevy_systems;
