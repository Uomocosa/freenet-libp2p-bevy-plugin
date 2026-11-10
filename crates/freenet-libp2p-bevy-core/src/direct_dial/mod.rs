#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::components::Friend;
pub use basic::enums::KnownPeerKind;
pub use basic::resources::{Config, DialEventTap, FriendAddressError};
pub use basic::structs::KnownPeer;

pub mod state_machine;

#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;

mod peer_book;
pub use peer_book::PeerBook;

mod peer_book_error;
pub use peer_book_error::PeerBookError;

mod friends_plugin;
pub use friends_plugin::FriendsPlugin;
