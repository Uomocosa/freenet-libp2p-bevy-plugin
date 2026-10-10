#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::newtypes::{PeerAddr, PeerId, RoomName, Topic};
pub use basic::resources::NetworkId;
pub use basic::structs::Peer;
