#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::components::{
    Connected, HeldBy, HeldRooms, HistoryHold, HistoryHolders, HoldOf, InRoom, KeepAlive, Me,
    RoomMembers,
};
pub use basic::resources::PeerEventTap;

mod peer;
pub use peer::Peer;

mod tracking_plugin;
pub use tracking_plugin::TrackingPlugin;

pub mod bevy_systems;
