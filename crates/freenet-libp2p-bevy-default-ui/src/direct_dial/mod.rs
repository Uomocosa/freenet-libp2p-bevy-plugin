#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::components::{
    AddFriendButton, CopyToast, FriendAddressInput, FriendErrorText, FriendList,
    FriendRemoveButton, FriendRow, FriendStatusDot, FriendsCountText, FriendsDrawer,
    FriendsDrawerBackdrop, FriendsDrawerCloseButton, FriendsDrawerPanel, FriendsDrawerSlide,
    OwnAddressChip, OwnAddressText,
};
pub use basic::constants::*;
pub use basic::messages::ToggleFriendsDrawer;

#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;

mod friends_drawer_state;
pub use friends_drawer_state::FriendsDrawerState;

mod friends_ui_plugin;
pub use friends_ui_plugin::FriendsUiPlugin;
