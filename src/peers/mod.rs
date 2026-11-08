#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::components::{
    Connected, HeldBy, HeldRooms, HistoryHold, HistoryHolders, HoldOf, InRoom, KeepAlive, Me, Peer,
    Room, RoomMembers,
};
pub use basic::resources::PeerEventTap;

mod peer_index;
pub use peer_index::PeerIndex;

mod room_index;
pub use room_index::RoomIndex;

mod tracking_plugin;
pub use tracking_plugin::TrackingPlugin;

pub mod bevy_systems;
