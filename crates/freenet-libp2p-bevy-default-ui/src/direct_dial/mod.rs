#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::components::{
    AddFriendButton, FriendAddressInput, FriendErrorText, FriendList, FriendRemoveButton,
    FriendRow, FriendsTab, OwnAddressText,
};

pub mod bevy_systems;

mod friends_ui_plugin;
pub use friends_ui_plugin::FriendsUiPlugin;
