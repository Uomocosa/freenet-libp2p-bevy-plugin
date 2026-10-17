#[cfg(feature = "discovery")]
pub mod client;
#[cfg(all(feature = "discovery", feature = "default_ui"))]
pub mod default_ui_plugin;
#[cfg(feature = "discovery")]
pub mod lobby;
pub mod p2p_plugin;
#[cfg(feature = "discovery")]
pub mod plugin;
