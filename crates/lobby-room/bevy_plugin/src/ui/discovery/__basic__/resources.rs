use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::discovery as core_discovery;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct SelectedRoom {
    pub room: Option<core_discovery::RoomName>,
    pub is_locked: bool,
}

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct JoinError(pub Option<String>);

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct FriendsPressBeat(pub f32);
