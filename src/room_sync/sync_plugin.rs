use std::marker::PhantomData;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::*;

use crate::identity;
use crate::room_sync;

pub struct SyncPlugin<S: room_sync::RoomState> {
    pub game: identity::GameId,
    pub marker: PhantomData<S>,
}

impl<S: room_sync::RoomState> SyncPlugin<S> {
    #[must_use]
    pub const fn new(game: identity::GameId) -> Self {
        Self {
            game,
            marker: PhantomData,
        }
    }
}

#[atomic_delegates]
impl<S: room_sync::RoomState> SyncPlugin<S> {
    pub fn build_plugin(&self, app: &mut App) {}
}

#[rustfmt::skip]
impl<S: room_sync::RoomState> bevy::prelude::Plugin for SyncPlugin<S> {
    fn build(&self, app: &mut App) {
        self.build_plugin(app);
    }
}

// no test_usage necessary
