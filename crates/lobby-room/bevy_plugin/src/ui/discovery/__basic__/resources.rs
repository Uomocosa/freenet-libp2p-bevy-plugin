use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::discovery as core_discovery;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct SelectedRoom(pub Option<core_discovery::RoomName>);

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct StreamedRoom(pub Option<core_discovery::RoomName>);

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct HoveredOccupancyPill(pub Option<core_discovery::RoomName>);

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref)]
pub struct PeekedRoomFriends(pub Vec<core_discovery::RoomFriend>);

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct JoinError(pub Option<String>);

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct FriendsPressBeat(pub f32);
