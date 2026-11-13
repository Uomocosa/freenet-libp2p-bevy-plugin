pub use freenet_libp2p_bevy_core::{
    direct_dial, identity, p2p, peers, plugin, presence, room_sync, time,
};

pub mod discovery;
pub mod freenet;
#[cfg(feature = "testing")]
pub mod testing;

#[path = "../__methods__/mod.rs"]
mod methods;

#[cfg(test)]
mod runner_config_tests;
