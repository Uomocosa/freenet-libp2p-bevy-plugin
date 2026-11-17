use bevy::prelude::{MessageWriter, Res, ResMut};

use crate::identity;
use crate::p2p;
use crate::replica_sync;
use crate::spectator;

pub fn route_topic_messages<S: replica_sync::traits::ReplicatedState>(
    game: Res<spectator::SpectatorGame>,
    events: Res<spectator::SpectatorEvents>,
    machine: Res<spectator::Machine<S::Op, Vec<u8>>>,
    mut bootstrap: ResMut<spectator::HistoryBootstrap<Vec<u8>>>,
    mut inputs: ResMut<spectator::SpectatorInputs<S::Op, Vec<u8>>>,
    spectated: Option<Res<spectator::Spectated<S>>>,
    mut network: MessageWriter<p2p::Libp2pCommand>,
) {
    let game = game.into_inner();
    let machine = machine.into_inner();
    let Ok(mut receiver) = events.into_inner().lock() else {
        return;
    };
    let spectated = spectated.map(Res::into_inner);
    while let Ok(event) = receiver.try_recv() {
        match event {
            p2p::Libp2pEvent::Ready(peer) => {
                inputs.push(spectator::state_machine::SpectateInput::LocalPeer(Some(
                    peer.id,
                )));
            }
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
                    replica_sync::ReplicaSyncTopicMessage::ApplyThisOp { ref zone, .. } => {
                        if machine.zone.as_ref() == Some(zone) {
                            inputs.push(spectator::state_machine::SpectateInput::Topic(message));
                        }
                    }
                    replica_sync::ReplicaSyncTopicMessage::CompareSliceContentHashes {
                        zone,
                        hashes,
                    } => {
                        if machine.zone.as_ref() != Some(&zone) {
                            continue;
                        }
                        let Some(spectated) = spectated else {
                            continue;
                        };
                        inputs.push(
                            spectator::state_machine::SpectateInput::RemoteSliceContentHashes {
                                from: from.clone(),
                                zone,
                                local: spectated.state.slice_content_hashes(),
                                remote: hashes,
                            },
                        );
                    }
                    replica_sync::ReplicaSyncTopicMessage::SpectatorLease {
                        zone,
                        spectator,
                        until,
                    } => {
                        inputs.push(spectator::state_machine::SpectateInput::Lease {
                            zone,
                            spectator,
                            until,
                        });
                    }
                }
            }
            p2p::Libp2pEvent::HistoryRecordFound { id, value } => {
                match bootstrap.handle_history_record(&id, value) {
                    spectator::BootstrapOutcome::RequestChunks(ids) => {
                        for id in ids {
                            network.write(p2p::Libp2pCommand::GetHistoryRecord { id });
                        }
                    }
                    spectator::BootstrapOutcome::Snapshot(bytes) => {
                        inputs.push(spectator::state_machine::SpectateInput::Snapshot(bytes));
                    }
                    spectator::BootstrapOutcome::Ignored => {}
                }
            }
            p2p::Libp2pEvent::ReplicaSlice {
                zone, index, data, ..
            } if machine.zone.as_ref() == Some(&zone) => {
                inputs.push(spectator::state_machine::SpectateInput::SliceReceived {
                    index,
                    snapshot: data,
                });
            }
            _ => {}
        }
    }
}

// no test_usage necessary
