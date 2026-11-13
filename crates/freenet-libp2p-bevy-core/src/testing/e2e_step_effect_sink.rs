use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use atomic_delegate_macros::atomic_delegates;

use crate::discovery;
use crate::testing;

#[derive(SystemParam)]
pub struct E2EStepEffectSink<'w> {
    pub(crate) pending_action: ResMut<'w, testing::PendingAction>,
    pub(crate) commands: MessageWriter<'w, discovery::Command>,
}

#[atomic_delegates]
impl E2EStepEffectSink<'_> {
    pub fn apply(&mut self, effect: testing::E2EStepEffect) {}
}

// no test_usage necessary
