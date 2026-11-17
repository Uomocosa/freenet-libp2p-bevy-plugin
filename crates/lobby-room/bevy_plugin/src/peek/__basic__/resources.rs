use bevy::prelude::{Handle, Image, Resource};
use derive_more::Deref;

use crate::identity;

#[derive(Resource, Debug, Deref)]
pub struct RoomPeekGame(pub identity::GameId);

#[derive(Resource, Debug, Clone)]
pub struct RoomPreviewTarget {
    pub zone: identity::ZoneName,
    pub image: Handle<Image>,
}

#[derive(Resource, Debug, Clone)]
pub struct RoomStreamTarget {
    pub zone: identity::ZoneName,
    pub image: Handle<Image>,
}

// no test_usage necessary
