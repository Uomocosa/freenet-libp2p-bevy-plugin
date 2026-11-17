use bevy::prelude::{MessageReader, Query, ResMut, With};

use crate::identity;
use crate::peers;
use crate::replica_sync;
use crate::spectator;

pub fn route_spectate_commands<S: replica_sync::traits::ReplicatedState>(
    mut commands: MessageReader<spectator::SpectateCommand>,
    me_zones: Query<&peers::InZone, With<peers::Me>>,
    zone_names: Query<&identity::ZoneName>,
    mut inputs: ResMut<spectator::SpectatorInputs<S::Op, Vec<u8>>>,
) {
    let my_zone = me_zones
        .iter()
        .next()
        .and_then(|in_zone| zone_names.get(**in_zone).ok())
        .cloned();
    inputs.push(spectator::MachineInput::MyZone(my_zone));
    for command in commands.read() {
        inputs.push(spectator::MachineInput::Command(command.clone()));
    }
}

// no test_usage necessary
