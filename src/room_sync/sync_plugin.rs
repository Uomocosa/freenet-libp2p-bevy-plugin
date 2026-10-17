use std::marker::PhantomData;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::*;

use crate::identity;
use crate::room_sync;

pub struct SyncPlugin<Replica: room_sync::RoomState> {
    pub game: identity::GameId,
    pub marker: PhantomData<Replica>,
}

impl<Replica: room_sync::RoomState> SyncPlugin<Replica> {
    #[must_use]
    pub const fn new(game: identity::GameId) -> Self {
        Self {
            game,
            marker: PhantomData,
        }
    }
}

#[atomic_delegates]
impl<Replica: room_sync::RoomState> SyncPlugin<Replica> {
    pub fn build_plugin(&self, app: &mut App) {}
}

#[rustfmt::skip]
impl<Replica: room_sync::RoomState> bevy::prelude::Plugin for SyncPlugin<Replica> {
    fn build(&self, app: &mut App) {
        self.build_plugin(app);
    }
}

// no test_usage necessary
