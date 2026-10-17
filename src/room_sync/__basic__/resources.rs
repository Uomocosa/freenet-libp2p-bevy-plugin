use bevy::prelude::{Resource, Timer};
use derive_more::{Deref, DerefMut};

use crate::identity;

#[derive(Resource, Deref)]
pub struct SyncGame(pub identity::GameId);

#[derive(Resource, Deref, DerefMut)]
pub struct AntiEntropyTimer(pub Timer);

// no test_usage necessary
