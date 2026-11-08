#[cfg(feature = "discovery")]
pub mod discovery;
#[cfg(feature = "discovery")]
pub mod freenet;

pub mod direct_dial;

pub mod identity;
#[path = "../methods/mod.rs"]
pub mod methods;
pub mod p2p;
pub mod plugin;
pub mod room_sync;
#[cfg(feature = "testing")]
pub mod testing;
