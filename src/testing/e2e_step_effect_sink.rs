use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::discovery;
use crate::testing;

#[derive(SystemParam)]
pub struct E2EStepEffectSink<'w> {
    pending_action: ResMut<'w, testing::PendingAction>,
    commands: MessageWriter<'w, discovery::Command>,
}

#[rustfmt::skip]
impl E2EStepEffectSink<'_> {
    pub fn apply(&mut self, effect: testing::E2EStepEffect) {
        match effect {
            testing::E2EStepEffect::ArmRoomAction(action) => {
                **self.pending_action = Some(action);
            }
            testing::E2EStepEffect::SendDiscoveryCommand(command) => {
                self.commands.write(command);
            }
            testing::E2EStepEffect::AbortProcess => {
                tracing::warn!(target: "script", "script crash-app: aborting the process");
                std::process::abort();
            }
        }
    }
}

// no test_usage necessary
