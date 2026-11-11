use bevy::prelude::Component;
use derive_more::{Deref, DerefMut};

use freenet_libp2p_bevy_core::identity;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct FriendsDrawer;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct FriendsDrawerBackdrop;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct FriendsDrawerPanel;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct FriendsDrawerCloseButton;

#[derive(Component, Debug, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct FriendsDrawerSlide(pub f32);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct OwnAddressChip;

#[derive(Component, Debug, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct CopyToast(pub f32);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct FriendStatusDot;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct FriendsCountText;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct FriendAddressInput;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct AddFriendButton;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
pub struct FriendRow(pub identity::PeerId);

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
pub struct FriendRemoveButton(pub identity::PeerId);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct FriendList;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct OwnAddressText;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct FriendErrorText;
