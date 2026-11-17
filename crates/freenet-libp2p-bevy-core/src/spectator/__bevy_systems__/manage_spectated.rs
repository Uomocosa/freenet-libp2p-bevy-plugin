use bevy::prelude::{Commands, Entity, MessageReader, Query};

use crate::replica_sync;
use crate::spectator;

pub fn manage_spectated<S: replica_sync::traits::ReplicatedState>(
    mut commands: Commands,
    mut spectate: MessageReader<spectator::SpectateCommand>,
    existing: Query<(Entity, &spectator::Spectated<S>)>,
) {
    for command in spectate.read() {
        match command {
            spectator::SpectateCommand::Start { slot, zone, .. } => {
                for (entity, spectated) in &existing {
                    if spectated.slot == *slot {
                        commands.entity(entity).despawn();
                    }
                }
                commands.spawn(spectator::Spectated {
                    slot: *slot,
                    zone: zone.clone(),
                    state: S::default(),
                    freshness: spectator::SpectateFreshness::Bootstrapping,
                });
            }
            spectator::SpectateCommand::Stop { slot } => {
                for (entity, spectated) in &existing {
                    if spectated.slot == *slot {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}

// no test_usage necessary
