use bevy::prelude::{MessageWriter, Query, Res, ResMut};

use crate::identity;
use crate::p2p;
use crate::replica_sync;
use crate::spectator;

pub fn route_topic_messages<S: replica_sync::traits::ReplicatedState>(
    game: Res<spectator::SpectatorGame>,
    events: Res<spectator::SpectatorEvents>,
    machine: Res<spectator::Machine<S::Op>>,
    mut bootstrap: ResMut<spectator::HistoryBootstrap<Vec<u8>>>,
    mut inputs: ResMut<spectator::SpectatorInputs<S::Op, Vec<u8>>>,
    spectated: Query<&spectator::Spectated<S>>,
    mut network: MessageWriter<p2p::Libp2pCommand>,
) {
    let game = game.into_inner();
    let machine = machine.into_inner();
    let Ok(mut receiver) = events.into_inner().lock() else {
        return;
    };
    while let Ok(event) = receiver.try_recv() {
        match event {
            p2p::Libp2pEvent::Ready(peer) => inputs.push(spectator::MachineInput::Session(
                spectator::state_machine::SpectateInput::LocalPeer(Some(peer.id)),
            )),
            p2p::Libp2pEvent::Gossip { topic, from, data } => {
                let Ok(message) =
                    bincode::deserialize::<replica_sync::ReplicaSyncTopicMessage<S::Op>>(&data)
                else {
                    continue;
                };
                let zone = match &message {
                    replica_sync::ReplicaSyncTopicMessage::ApplyThisOp { zone, .. }
                    | replica_sync::ReplicaSyncTopicMessage::CompareSliceContentHashes {
                        zone,
                        ..
                    }
                    | replica_sync::ReplicaSyncTopicMessage::SpectatorLease { zone, .. } => zone,
                };
                if identity::Topic::from_replica_sync(game, zone) != topic {
                    continue;
                }
                match message {
                    replica_sync::ReplicaSyncTopicMessage::ApplyThisOp { .. } => {
                        inputs.push(spectator::MachineInput::Session(
                            spectator::state_machine::SpectateInput::Topic(message),
                        ));
                    }
                    replica_sync::ReplicaSyncTopicMessage::CompareSliceContentHashes {
                        zone,
                        hashes,
                    } => {
                        let Some(seen) = spectated.iter().find(|seen| seen.zone == zone) else {
                            continue;
                        };
                        inputs.push(spectator::MachineInput::Session(
                            spectator::state_machine::SpectateInput::RemoteSliceContentHashes {
                                from: from.clone(),
                                zone,
                                local: seen.state.slice_content_hashes(),
                                remote: hashes,
                            },
                        ));
                    }
                    replica_sync::ReplicaSyncTopicMessage::SpectatorLease {
                        zone,
                        spectator: peer,
                        until,
                    } => inputs.push(spectator::MachineInput::Lease {
                        zone,
                        spectator: peer,
                        until,
                    }),
                }
            }
            p2p::Libp2pEvent::HistoryRecordFound { id, value } => {
                match bootstrap.handle_history_record(&id, value) {
                    spectator::BootstrapOutcome::RequestChunks(ids) => {
                        for id in ids {
                            network.write(p2p::Libp2pCommand::GetHistoryRecord { id });
                        }
                    }
                    spectator::BootstrapOutcome::Snapshot {
                        history_prefix,
                        snapshot,
                    } => inputs.push(spectator::MachineInput::Session(
                        spectator::state_machine::SpectateInput::Snapshot {
                            history_prefix,
                            snapshot,
                        },
                    )),
                    spectator::BootstrapOutcome::Ignored => {}
                }
            }
            p2p::Libp2pEvent::ReplicaSlice {
                zone, index, data, ..
            } if machine.does_any_slot_watch(&zone) => {
                inputs.push(spectator::MachineInput::Session(
                    spectator::state_machine::SpectateInput::SliceReceived {
                        zone,
                        index,
                        snapshot: data,
                    },
                ));
            }
            _ => {}
        }
    }
}
