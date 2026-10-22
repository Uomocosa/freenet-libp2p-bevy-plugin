use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::room_sync;

#[derive(Resource, Deref, DerefMut)]
pub struct SyncState<Op>(pub room_sync::state_machine::ReplicaState<Op>);

impl<Op> Default for SyncState<Op> {
    fn default() -> Self {
        Self(room_sync::state_machine::ReplicaState::default())
    }
}
