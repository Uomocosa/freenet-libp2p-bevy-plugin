use std::marker::PhantomData;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::*;

use crate::identity;
use crate::room_sync;

pub struct SyncPlugin<Replica: room_sync::RoomState> {
    pub game: identity::GameId,
    pub anti_entropy: Option<room_sync::AntiEntropyConfig>,
    pub history: Option<room_sync::HistoryConfig>,
    pub marker: PhantomData<Replica>,
}

impl<Replica: room_sync::RoomState> SyncPlugin<Replica> {
    #[must_use]
    pub fn new(game: identity::GameId) -> Self {
        Self {
            game,
            anti_entropy: Some(room_sync::AntiEntropyConfig::default()),
            history: None,
            marker: PhantomData,
        }
    }

    #[must_use]
    pub const fn with_anti_entropy_config(
        mut self,
        anti_entropy: Option<room_sync::AntiEntropyConfig>,
    ) -> Self {
        self.anti_entropy = anti_entropy;
        self
    }

    #[must_use]
    pub const fn with_history_config(mut self, history: Option<room_sync::HistoryConfig>) -> Self {
        self.history = history;
        self
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
