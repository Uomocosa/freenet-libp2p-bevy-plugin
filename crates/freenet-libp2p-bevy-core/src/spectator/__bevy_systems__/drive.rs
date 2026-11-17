use bevy::prelude::{Commands, Has, MessageWriter, Query, Res, ResMut};

use crate::identity;
use crate::p2p;
use crate::peers;
use crate::replica_sync;
use crate::spectator;
use crate::time;

pub fn drive<S: replica_sync::traits::ReplicatedState>(
    game: Res<spectator::SpectatorGame>,
    mut machine: ResMut<spectator::SpectatorMachine<S::Op, Vec<u8>>>,
    mut inputs: ResMut<spectator::SpectatorInputs<S::Op, Vec<u8>>>,
    mut bootstrap: ResMut<spectator::HistoryBootstrap<Vec<u8>>>,
    mut spectated: Option<ResMut<spectator::Spectated<S>>>,
    mut count: ResMut<spectator::SpectatorCount>,
    mut commands: Commands,
    mut network: MessageWriter<p2p::Libp2pCommand>,
    players: Query<(&peers::Peer, Has<peers::InZone>, Has<peers::Me>)>,
) {
    use p2p::Libp2pCommand;
    use spectator::state_machine::SpectateOutput;

    let game = game.into_inner();
    let now = time::UnixTime::now();
    let players: Vec<identity::PeerId> = players
        .iter()
        .filter(|(_, is_in_zone, is_me)| *is_in_zone && !*is_me)
        .map(|(peer, _, _)| (**peer).clone())
        .collect();
    inputs.push(spectator::state_machine::SpectateInput::Players(players));
    let queued = std::mem::take(&mut **inputs);
    for input in queued {
        for output in machine.update(input, now) {
            match output {
                SpectateOutput::Subscribe(topic) => {
                    if let Some(zone) = machine.zone.clone()
                        && topic == identity::Topic::from_replica_sync(game, &zone)
                    {
                        bootstrap.reset_for_spectated::<S>(&mut commands, zone);
                    }
                    network.write(Libp2pCommand::Subscribe { topic });
                }
                SpectateOutput::Unsubscribe(topic) => {
                    network.write(Libp2pCommand::Unsubscribe { topic });
                }
                SpectateOutput::GetHistoryRecord(id) => {
                    network.write(Libp2pCommand::GetHistoryRecord { id });
                }
                SpectateOutput::RequestSlice { from, index } => {
                    let Some(zone) = machine.zone.clone() else {
                        continue;
                    };
                    if let Some(spectated) = spectated.as_mut() {
                        spectated.freshness = spectator::SpectateFreshness::Repairing;
                    }
                    network.write(Libp2pCommand::FetchReplicaSlice {
                        peer_id: from,
                        zone,
                        index,
                    });
                }
                SpectateOutput::Publish(message) => {
                    let Some(zone) = machine.zone.clone() else {
                        continue;
                    };
                    if let Ok(data) = bincode::serialize(&message) {
                        network.write(Libp2pCommand::Publish {
                            topic: identity::Topic::from_replica_sync(game, &zone),
                            data,
                        });
                    }
                }
                SpectateOutput::PublishLease {
                    zone,
                    spectator,
                    until,
                } => {
                    let message = replica_sync::ReplicaSyncTopicMessage::<S::Op>::SpectatorLease {
                        zone: zone.clone(),
                        spectator,
                        until,
                    };
                    if let Ok(data) = bincode::serialize(&message) {
                        network.write(Libp2pCommand::Publish {
                            topic: identity::Topic::from_replica_sync(game, &zone),
                            data,
                        });
                    }
                }
                SpectateOutput::Merge(bytes) => {
                    if let Some(spectated) = spectated.as_mut()
                        && let Ok(snapshot) = bincode::deserialize::<S::Snapshot>(&bytes)
                    {
                        spectated.state.merge(snapshot);
                        spectated.freshness = spectator::SpectateFreshness::Live;
                    }
                }
                SpectateOutput::Apply { meta, op } => {
                    if let Some(spectated) = spectated.as_mut() {
                        spectated.state.apply(meta, &op);
                    }
                }
                SpectateOutput::Clear => {
                    commands.remove_resource::<spectator::Spectated<S>>();
                    bootstrap.reset();
                }
                SpectateOutput::RepairFinished => {
                    if let Some(spectated) = spectated.as_mut() {
                        spectated.freshness = spectator::SpectateFreshness::Live;
                    }
                }
                SpectateOutput::SpectatorCountChanged(spectators) => {
                    **count = spectators;
                }
            }
        }
    }
}

// no test_usage necessary
