pub use freenet_libp2p_bevy_core::{
    direct_dial, identity, p2p, peers, plugin, presence, replica_sync, spectator, time,
};

pub mod discovery;
pub mod freenet;
mod lobby_room_plugins;
pub mod peek;
#[cfg(feature = "testing")]
pub mod testing;
pub mod ui;
pub use lobby_room_plugins::LobbyRoomPlugins;

#[path = "../__methods__/mod.rs"]
mod methods;
