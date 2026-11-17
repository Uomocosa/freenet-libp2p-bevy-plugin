use bevy::prelude::{MessageReader, ResMut};

use crate::replica_sync;
use crate::spectator;

pub fn route_spectate_commands<S: replica_sync::traits::ReplicatedState>(
    mut commands: MessageReader<spectator::SpectateCommand>,
    mut inputs: ResMut<spectator::SpectatorInputs<S::Op, Vec<u8>>>,
) {
    for command in commands.read() {
        inputs.push(spectator::state_machine::SpectateInput::Command(
            command.clone(),
        ));
    }
}

// no test_usage necessary
