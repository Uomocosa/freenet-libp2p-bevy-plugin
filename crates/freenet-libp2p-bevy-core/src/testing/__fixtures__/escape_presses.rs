use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct EscapePresses(pub u32);
