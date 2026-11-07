#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::enums::KnownPeerKind;
pub use basic::resources::{Config, FriendAddressError, FriendEventTap};
pub use basic::structs::KnownPeer;

mod connected_peers;
pub use connected_peers::ConnectedPeers;

mod peer_book;
pub use peer_book::PeerBook;

mod peer_book_error;
pub use peer_book_error::PeerBookError;

mod plugin;
pub use plugin::Plugin;

#[cfg(feature = "default_ui")]
pub mod ui;
