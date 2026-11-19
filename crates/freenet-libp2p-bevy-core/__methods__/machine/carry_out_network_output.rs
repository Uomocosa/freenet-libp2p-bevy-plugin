use bevy::prelude::MessageWriter;

use crate::identity;
use crate::p2p;
use crate::replica_sync;
use crate::spectator;

pub fn carry_out_network_output<S: replica_sync::traits::ReplicatedState>(
    machine: &spectator::Machine<S::Op>,
    slot: spectator::SpectateSlot,
    output: &spectator::state_machine::SpectateOutput<S::Op, Vec<u8>>,
    game: &spectator::SpectatorGame,
    network: &mut MessageWriter<p2p::Libp2pCommand>,
) {
    use p2p::Libp2pCommand;
    use spectator::state_machine::SpectateOutput;
    match output {
        SpectateOutput::Subscribe(topic) => {
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
            let Some(zone) = machine.zone_of(slot) else {
                return;
            };
            network.write(Libp2pCommand::FetchReplicaSlice {
                peer_id: from.clone(),
                zone: zone.clone(),
                index: *index,
            });
        }
        SpectateOutput::Publish(message) => {
            let Some(zone) = machine.zone_of(slot) else {
                return;
            };
            if let Ok(data) = bincode::serialize(message) {
                network.write(Libp2pCommand::Publish {
                    topic: identity::Topic::from_replica_sync(game, zone),
                    data,
                });
            }
        }
        SpectateOutput::PublishLease {
            zone,
            spectator: peer,
            until,
        } => {
            let message = replica_sync::ReplicaSyncTopicMessage::<S::Op>::SpectatorLease {
                zone: zone.clone(),
                spectator: peer.clone(),
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
        | SpectateOutput::RepairFinished => {}
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;

    use crate::identity;
    use crate::p2p;
    use crate::spectator;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        app.add_message::<p2p::Libp2pCommand>();
        app.insert_resource(testing::machine());
        app.insert_resource(spectator::SpectatorGame(identity::GameId::from("game")));
        let zone = identity::ZoneName::from("zone");
        app.world_mut()
            .run_system_once_with(
                |input: In<spectator::state_machine::SpectateOutput<u8, Vec<u8>>>,
                 machine: Res<spectator::Machine<u8>>,
                 game: Res<spectator::SpectatorGame>,
                 mut network: MessageWriter<p2p::Libp2pCommand>| {
                    let In(output) = input;
                    machine.carry_out_network_output::<testing::FakeMachineState>(
                        spectator::SpectateSlot::Preview,
                        &output,
                        &game,
                        &mut network,
                    );
                },
                spectator::state_machine::SpectateOutput::<u8, Vec<u8>>::WatchZone(zone.clone()),
            )
            .expect("run");
        assert_eq!(
            testing::written_messages::<p2p::Libp2pCommand>(&mut app).expect("written"),
            vec![p2p::Libp2pCommand::WatchZone { zone }]
        );
    }

    #[test]
    fn state_outputs_write_nothing() {
        let mut app = testing::TestApp::new();
        app.add_message::<p2p::Libp2pCommand>();
        app.insert_resource(testing::machine());
        app.insert_resource(spectator::SpectatorGame(identity::GameId::from("game")));
        app.world_mut()
            .run_system_once_with(
                |input: In<spectator::state_machine::SpectateOutput<u8, Vec<u8>>>,
                 machine: Res<spectator::Machine<u8>>,
                 game: Res<spectator::SpectatorGame>,
                 mut network: MessageWriter<p2p::Libp2pCommand>| {
                    let In(output) = input;
                    machine.carry_out_network_output::<testing::FakeMachineState>(
                        spectator::SpectateSlot::Preview,
                        &output,
                        &game,
                        &mut network,
                    );
                },
                spectator::state_machine::SpectateOutput::<u8, Vec<u8>>::RepairFinished,
            )
            .expect("run");
        assert_eq!(
            testing::written_messages::<p2p::Libp2pCommand>(&mut app).expect("written"),
            Vec::new()
        );
    }
}
