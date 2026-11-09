use bevy::prelude::Component;
use derive_more::Deref;

use freenet_libp2p_bevy_core::identity;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct FriendsTab;

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
