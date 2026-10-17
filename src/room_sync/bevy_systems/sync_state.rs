use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::room_sync;

#[derive(Resource, Deref, DerefMut)]
pub struct SyncState<O>(pub room_sync::state_machine::State<O>);

impl<O> Default for SyncState<O> {
    fn default() -> Self {
        Self(room_sync::state_machine::State::default())
    }
}
