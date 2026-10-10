#[cfg(feature = "room_lobby")]
pub mod discovery;

#[path = "../methods/mod.rs"]
pub mod methods;
pub mod net_id;
pub mod p2p;
pub mod plugin;
pub mod roster;
