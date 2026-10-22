use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

use crate::shared::intent::PendingAction;
use crate::shared::script::poll_shared_script_step::SharedStepEffect;

/// The parts of the app a shared step's effect reaches.
#[derive(SystemParam)]
pub struct SharedStepEffectSink<'w> {
    pending_action: ResMut<'w, PendingAction>,
    commands: MessageWriter<'w, discovery::Command>,
}

impl SharedStepEffectSink<'_> {
    /// Performs a shared step's effect on the running app.
    pub fn apply(&mut self, effect: SharedStepEffect) {
        match effect {
            SharedStepEffect::ArmRoomAction(action) => self.pending_action.0 = Some(action),
            SharedStepEffect::SendDiscoveryCommand(command) => {
                self.commands.write(command);
            }
            SharedStepEffect::AbortProcess => {
                tracing::warn!("script crash-app: aborting the process");
                std::process::abort();
            }
        }
    }
}

// no test_usage necessary: a thin adapter from SharedStepEffect to the app; the effects are
// covered by poll_shared_script_step's tests and the example script_runner tests
