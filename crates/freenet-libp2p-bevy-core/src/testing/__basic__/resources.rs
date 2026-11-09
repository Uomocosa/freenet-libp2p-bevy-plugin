use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::testing;

#[derive(Resource, Debug, Default, Clone, Deref, DerefMut)]
pub struct PendingAction(pub Option<testing::Action>);
