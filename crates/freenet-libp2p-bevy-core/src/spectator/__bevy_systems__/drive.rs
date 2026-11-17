use bevy::prelude::{Has, MessageWriter, Query, Res, ResMut};

use crate::identity;
use crate::p2p;
use crate::peers;
use crate::replica_sync;
use crate::spectator;
use crate::time;

pub fn drive<S: replica_sync::traits::ReplicatedState>(
    game: Res<spectator::SpectatorGame>,
    mut machine: ResMut<spectator::Machine<S::Op>>,
    mut inputs: ResMut<spectator::SpectatorInputs<S::Op, Vec<u8>>>,
    mut count: ResMut<spectator::SpectatorCount>,
    mut bootstrap: ResMut<spectator::HistoryBootstrap<Vec<u8>>>,
    mut network: MessageWriter<p2p::Libp2pCommand>,
    mut spectated: Query<&mut spectator::Spectated<S>>,
    players: Query<(&peers::Peer, Has<peers::InZone>, Has<peers::Me>)>,
) {
    let game = game.into_inner();
    let now = time::UnixTime::now();
    let players: Vec<identity::PeerId> = players
        .iter()
        .filter(|(_, is_in_zone, is_me)| *is_in_zone && !*is_me)
        .map(|(peer, _, _)| (**peer).clone())
        .collect();
    inputs.push(spectator::MachineInput::Session(
        spectator::state_machine::SpectateInput::Players(players),
    ));
    for input in std::mem::take(&mut **inputs) {
        for output in machine.update(input, now) {
            match output {
                spectator::MachineOutput::Session {
                    slot,
                    output: session_output,
                } => {
                    machine.carry_out_network_output::<S>(
                        slot,
                        &session_output,
                        game,
                        &mut network,
                    );
                    let target = spectated
                        .iter_mut()
                        .find(|spectated| spectated.slot == slot)
                        .map(bevy::ecs::change_detection::Mut::into_inner);
                    spectator::Machine::<S::Op>::carry_out_state_output(&session_output, target);
                }
                spectator::MachineOutput::SpectatorCountChanged(spectators) => {
                    **count = spectators;
                }
            }
        }
    }
    let live_prefixes: Vec<&p2p::HistoryKeyPrefix> = machine
        .sessions
        .values()
        .filter_map(|session| session.history_prefix.as_ref())
        .collect();
    bootstrap
        .pending
        .retain(|prefix, _| live_prefixes.contains(&prefix));
}

// no test_usage necessary
