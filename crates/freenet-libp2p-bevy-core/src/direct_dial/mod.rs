#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::components::Friend;
pub use basic::enums::{FriendPresence, KnownPeerKind};
pub use basic::messages::FriendAddressSubmitted;
pub use basic::resources::{Config, DialEventTap, FriendAddressError, OwnShareAddress};
pub use basic::structs::{FriendView, KnownPeer};

mod own_address;
pub use own_address::OwnAddress;

mod friends_view;
pub use friends_view::FriendsView;

pub mod state_machine;

#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;

mod peer_book;
pub use peer_book::PeerBook;

mod peer_book_error;
pub use peer_book_error::PeerBookError;

mod friends_plugin;
pub use friends_plugin::FriendsPlugin;
