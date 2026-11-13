#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::constants::*;
pub use basic::messages::Output;
pub use basic::structs::{
    Candidate, Handshake, HeartbeatTarget, HeldRoomHistory, ObservedLobbyHeartbeat,
};

mod input;
pub use input::Input;

mod output;

mod discovery_state;
pub use discovery_state::DiscoveryState;
