use bevy::prelude::ResMut;

use crate::replica_sync;
use crate::spectator;

pub fn count_spectators<S: replica_sync::traits::ReplicatedState>(
    mut inputs: ResMut<spectator::SpectatorInputs<S::Op, Vec<u8>>>,
) {
    inputs.push(spectator::MachineInput::Session(
        spectator::state_machine::SpectateInput::Tick,
    ));
}
