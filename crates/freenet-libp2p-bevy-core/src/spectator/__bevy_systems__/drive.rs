use bevy::prelude::{Commands, Has, MessageWriter, Query, Res, ResMut};

use crate::identity;
use crate::p2p;
use crate::peers;
use crate::replica_sync;
use crate::spectator;
use crate::time;

pub fn drive<S: replica_sync::traits::ReplicatedState>(
    game: Res<spectator::SpectatorGame>,
    mut machine: ResMut<spectator::Machine<S::Op, Vec<u8>>>,
    mut inputs: ResMut<spectator::SpectatorInputs<S::Op, Vec<u8>>>,
    mut bootstrap: ResMut<spectator::HistoryBootstrap<Vec<u8>>>,
    mut spectated: Option<ResMut<spectator::Spectated<S>>>,
    mut count: ResMut<spectator::SpectatorCount>,
    mut commands: Commands,
    mut network: MessageWriter<p2p::Libp2pCommand>,
    players: Query<(&peers::Peer, Has<peers::InZone>, Has<peers::Me>)>,
) {
    let game = game.into_inner();
    let now = time::UnixTime::now();
    let players: Vec<identity::PeerId> = players
        .iter()
        .filter(|(_, is_in_zone, is_me)| *is_in_zone && !*is_me)
        .map(|(peer, _, _)| (**peer).clone())
        .collect();
    inputs.push(spectator::state_machine::SpectateInput::Players(players));
    for input in std::mem::take(&mut **inputs) {
        for output in machine.update(input, now) {
            machine.carry_out_network_output(
                &output,
                game,
                spectated.as_deref_mut(),
                &mut commands,
                &mut bootstrap,
                &mut network,
            );
            spectator::Machine::<S::Op, Vec<u8>>::carry_out_state_output(
                &output,
                spectated.as_deref_mut(),
                &mut count,
                &mut commands,
                &mut bootstrap,
            );
        }
    }
}

// no test_usage necessary
