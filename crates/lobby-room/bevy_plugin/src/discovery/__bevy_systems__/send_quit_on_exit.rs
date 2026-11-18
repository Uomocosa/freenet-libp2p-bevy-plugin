use bevy::prelude::*;

use crate::discovery;

pub fn send_quit_on_exit(
    mut exits: MessageReader<AppExit>,
    mut inputs: MessageWriter<discovery::state_machine::Input>,
) {
    if exits.read().next().is_some() {
        inputs.write(discovery::state_machine::Input::Command(
            discovery::Command::Quit,
        ));
    }
}
