use bevy::prelude::{Commands, MessageWriter};

use crate::identity;
use crate::p2p;
use crate::replica_sync;
use crate::spectator;

pub fn carry_out_network_output<S: replica_sync::traits::ReplicatedState>(
    machine: &spectator::Machine<S::Op, Vec<u8>>,
    output: &spectator::state_machine::SpectateOutput<S::Op, Vec<u8>>,
    game: &spectator::SpectatorGame,
    spectated: Option<&mut spectator::Spectated<S>>,
    commands: &mut Commands,
    bootstrap: &mut spectator::HistoryBootstrap<Vec<u8>>,
    network: &mut MessageWriter<p2p::Libp2pCommand>,
) {
    use p2p::Libp2pCommand;
    use spectator::state_machine::SpectateOutput;
    match output {
        SpectateOutput::Subscribe(topic) => {
            if let Some(zone) = machine.zone.clone()
                && *topic == identity::Topic::from_replica_sync(game, &zone)
            {
                bootstrap.reset_for_spectated::<S>(commands, zone);
            }
            network.write(Libp2pCommand::Subscribe {
                topic: topic.clone(),
            });
        }
        SpectateOutput::Unsubscribe(topic) => {
            network.write(Libp2pCommand::Unsubscribe {
                topic: topic.clone(),
            });
        }
        SpectateOutput::WatchZone(zone) => {
            network.write(Libp2pCommand::WatchZone { zone: zone.clone() });
        }
        SpectateOutput::UnwatchZone(zone) => {
            network.write(Libp2pCommand::UnwatchZone { zone: zone.clone() });
        }
        SpectateOutput::GetHistoryRecord(id) => {
            network.write(Libp2pCommand::GetHistoryRecord { id: id.clone() });
        }
        SpectateOutput::RequestSlice { from, index } => {
            let Some(zone) = machine.zone.clone() else {
                return;
            };
            if let Some(spectated) = spectated {
                spectated.freshness = spectator::SpectateFreshness::Repairing;
            }
            network.write(Libp2pCommand::FetchReplicaSlice {
                peer_id: from.clone(),
                zone,
                index: *index,
            });
        }
        SpectateOutput::Publish(message) => {
            let Some(zone) = machine.zone.clone() else {
                return;
            };
            if let Ok(data) = bincode::serialize(message) {
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
                spectator: spectator.clone(),
                until: *until,
            };
            if let Ok(data) = bincode::serialize(&message) {
                network.write(Libp2pCommand::Publish {
                    topic: identity::Topic::from_replica_sync(game, zone),
                    data,
                });
            }
        }
        SpectateOutput::Merge(_)
        | SpectateOutput::Apply { .. }
        | SpectateOutput::Clear
        | SpectateOutput::RepairFinished
        | SpectateOutput::SpectatorCountChanged(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;

    use crate::identity;
    use crate::p2p;
    use crate::replica_sync;
    use crate::spectator;

    #[derive(Default, Resource)]
    struct MockState;

    impl replica_sync::traits::ReplicatedState for MockState {
        type Op = u8;
        type Snapshot = u8;

        fn apply(&mut self, _meta: replica_sync::OpMeta, _op: &u8) {}

        fn snapshot(&self) -> u8 {
            0
        }

        fn merge(&mut self, _snapshot: u8) {}

        fn slice_content_hashes(&self) -> replica_sync::ReplicaSliceContentHashes {
            Vec::new()
        }

        fn slice_snapshot(&self, _index: replica_sync::ReplicaSliceIndex) -> u8 {
            0
        }
    }

    fn machine() -> spectator::Machine<u8, Vec<u8>> {
        spectator::Machine(spectator::state_machine::SpectatorState::new(
            identity::GameId::from("game"),
        ))
    }

    fn written(app: &mut App) -> Vec<p2p::Libp2pCommand> {
        app.world_mut()
            .run_system_once(|mut reader: MessageReader<p2p::Libp2pCommand>| {
                reader.read().cloned().collect::<Vec<_>>()
            })
            .unwrap()
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<p2p::Libp2pCommand>();
        app.insert_resource(machine());
        app.insert_resource(spectator::SpectatorGame(identity::GameId::from("game")));
        app.init_resource::<spectator::HistoryBootstrap<Vec<u8>>>();
        let zone = identity::ZoneName::from("zone");
        app.world_mut()
            .run_system_once_with(
                |input: In<spectator::state_machine::SpectateOutput<u8, Vec<u8>>>,
                 machine: Res<spectator::Machine<u8, Vec<u8>>>,
                 game: Res<spectator::SpectatorGame>,
                 mut bootstrap: ResMut<spectator::HistoryBootstrap<Vec<u8>>>,
                 mut commands: Commands,
                 mut network: MessageWriter<p2p::Libp2pCommand>| {
                    let In(output) = input;
                    machine.carry_out_network_output::<MockState>(
                        &output,
                        &game,
                        None,
                        &mut commands,
                        &mut bootstrap,
                        &mut network,
                    );
                },
                spectator::state_machine::SpectateOutput::<u8, Vec<u8>>::WatchZone(zone.clone()),
            )
            .unwrap();
        assert_eq!(
            written(&mut app),
            vec![p2p::Libp2pCommand::WatchZone { zone }]
        );
    }

    #[test]
    fn state_outputs_write_nothing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<p2p::Libp2pCommand>();
        app.insert_resource(machine());
        app.insert_resource(spectator::SpectatorGame(identity::GameId::from("game")));
        app.init_resource::<spectator::HistoryBootstrap<Vec<u8>>>();
        app.world_mut()
            .run_system_once_with(
                |input: In<spectator::state_machine::SpectateOutput<u8, Vec<u8>>>,
                 machine: Res<spectator::Machine<u8, Vec<u8>>>,
                 game: Res<spectator::SpectatorGame>,
                 mut bootstrap: ResMut<spectator::HistoryBootstrap<Vec<u8>>>,
                 mut commands: Commands,
                 mut network: MessageWriter<p2p::Libp2pCommand>| {
                    let In(output) = input;
                    machine.carry_out_network_output::<MockState>(
                        &output,
                        &game,
                        None,
                        &mut commands,
                        &mut bootstrap,
                        &mut network,
                    );
                },
                spectator::state_machine::SpectateOutput::<u8, Vec<u8>>::Clear,
            )
            .unwrap();
        assert_eq!(written(&mut app), Vec::new());
    }
}
