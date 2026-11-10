#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::components::{
    AddFriendButton, FriendAddressInput, FriendErrorText, FriendList, FriendRemoveButton,
    FriendRow, FriendsTab, OwnAddressText,
};

#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;

mod friends_ui_plugin;
pub use friends_ui_plugin::FriendsUiPlugin;
