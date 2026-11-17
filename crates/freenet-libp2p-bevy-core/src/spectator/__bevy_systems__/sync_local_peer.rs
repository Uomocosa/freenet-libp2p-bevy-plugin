use bevy::prelude::{Query, Res, ResMut, With};

use crate::peers;
use crate::replica_sync;
use crate::spectator;

pub fn sync_local_peer<S: replica_sync::traits::ReplicatedState>(
    machine: Res<spectator::SpectatorMachine<S::Op, Vec<u8>>>,
    mut inputs: ResMut<spectator::SpectatorInputs<S::Op, Vec<u8>>>,
    me: Query<&peers::Peer, With<peers::Me>>,
) {
    let machine = machine.into_inner();
    let peer = me.iter().next().map(|peer| (**peer).clone());
    if machine.me != peer {
        inputs.push(spectator::state_machine::SpectateInput::LocalPeer(peer));
    }
}

// no test_usage necessary
