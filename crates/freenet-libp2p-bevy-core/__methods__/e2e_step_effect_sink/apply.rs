use crate::testing;

pub fn apply(sink: &mut testing::E2EStepEffectSink<'_>, effect: testing::E2EStepEffect) {
    match effect {
        testing::E2EStepEffect::ArmRoomAction(action) => {
            **sink.pending_action = Some(action);
        }
        testing::E2EStepEffect::SendDiscoveryCommand(command) => {
            sink.commands.write(command);
        }
        testing::E2EStepEffect::AbortProcess => {
            tracing::warn!(target: "script", "script crash-app: aborting the process");
            std::process::abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::apply;
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    fn arm_room(mut sink: testing::E2EStepEffectSink) {
        let action = testing::Action::Create(identity::RoomName::from("room-1"));
        apply(&mut sink, testing::E2EStepEffect::ArmRoomAction(action));
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.init_resource::<testing::PendingAction>();
        app.add_message::<discovery::Command>();
        app.add_systems(Update, arm_room);
        app.update();
        assert!(app.world().resource::<testing::PendingAction>().is_some());
    }
}
