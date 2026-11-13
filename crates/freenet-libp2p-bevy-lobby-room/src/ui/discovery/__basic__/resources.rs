use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct JoinError(pub Option<String>);

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct FriendsPressBeat(pub f32);
