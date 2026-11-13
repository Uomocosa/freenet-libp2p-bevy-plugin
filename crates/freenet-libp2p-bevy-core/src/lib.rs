#[cfg(feature = "discovery")]
pub mod discovery;
#[cfg(feature = "discovery")]
pub mod freenet;

pub mod direct_dial;

pub mod identity;
#[path = "../__methods__/mod.rs"]
mod methods;
pub mod p2p;
pub mod peers;
pub mod plugin;
pub mod presence;
pub mod room_sync;
#[cfg(feature = "testing")]
pub mod testing;
pub mod time;
