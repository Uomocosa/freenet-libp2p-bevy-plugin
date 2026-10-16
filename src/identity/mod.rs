#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::newtypes::{FreenetPeerId, GameId, PeerAddr, PeerId, RoomName, Topic, Username};
pub use basic::structs::{FreenetPeer, Peer};
