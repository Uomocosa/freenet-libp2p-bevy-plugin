#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::components::Friend;
pub use basic::enums::KnownPeerKind;
pub use basic::resources::{Config, DialEventTap, FriendAddressError, FriendEventTap};
pub use basic::structs::KnownPeer;

mod connected_peers;
pub use connected_peers::ConnectedPeers;

pub mod state_machine;

pub mod bevy_systems;

mod peer_book;
pub use peer_book::PeerBook;

mod peer_book_error;
pub use peer_book_error::PeerBookError;

mod friends_plugin;
pub use friends_plugin::FriendsPlugin;

#[cfg(feature = "default_ui")]
pub mod ui;
