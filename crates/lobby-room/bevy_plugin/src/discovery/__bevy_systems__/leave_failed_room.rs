use bevy::prelude::*;

use crate::discovery;
use crate::replica_sync;

pub fn leave_failed_room(
    failures: Option<MessageReader<replica_sync::JoinFailure>>,
    mut inputs: MessageWriter<discovery::state_machine::Input>,
) {
    let Some(mut failures) = failures else {
        return;
    };
    if failures.read().next().is_some() {
        inputs.write(discovery::state_machine::Input::Command(
            discovery::Command::Leave,
        ));
    }
}
