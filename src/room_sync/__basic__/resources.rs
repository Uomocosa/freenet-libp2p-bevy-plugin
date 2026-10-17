use bevy::prelude::Resource;
use derive_more::Deref;

use crate::identity;

#[derive(Resource, Deref)]
pub struct RoomSyncGame(pub identity::GameId);

// no test_usage necessary
