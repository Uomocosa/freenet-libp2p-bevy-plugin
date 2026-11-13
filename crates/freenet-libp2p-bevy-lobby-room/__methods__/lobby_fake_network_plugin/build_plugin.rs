use bevy::prelude::{App, IntoScheduleConfigs, Update};
use lele_bevy_state_machine::StateMachineSystems;

use crate::discovery;
use crate::testing;

pub fn build_plugin(plugin: &testing::LobbyFakeNetworkPlugin, app: &mut App) {
    app.add_systems(
        Update,
        testing::bevy_systems::decode_extension_effects.before(
            StateMachineSystems::collect_inputs::<discovery::state_machine::DiscoveryState>(),
        ),
    );
    match plugin.network {
        testing::Network::Mainnet | testing::Network::LocalGateways => {}
        testing::Network::DirectDial => {
            app.insert_resource(discovery::LobbyTaskOverride {
                orchestrator_port: None,
                peer: plugin.peer.clone(),
            });
        }
        testing::Network::Fake => {
            let Some(port) = plugin.orchestrator_port else {
                return;
            };
            app.insert_resource(discovery::LobbyTaskOverride {
                orchestrator_port: Some(port),
                peer: plugin.peer.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::testing;

    #[test]
    fn test_usage() {
        let args = testing::NetworkArgs {
            network: testing::Network::Mainnet,
            orchestrator_port: None,
        };
        let plugin = testing::FakeNetworkPlugin::<u8>::from(&args);
        let mut app = bevy::prelude::App::new();
        super::build_plugin(&testing::LobbyFakeNetworkPlugin::new(&plugin), &mut app);
    }
}

#[cfg(test)]
mod proof_tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use bevy::prelude::*;
    use derive_more::{Deref, DerefMut};

    use super::build_plugin;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::plugin;
    use crate::room_sync;
    use crate::testing;

    #[test]
    fn direct_dial_keeps_an_idle_lobby_override() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let args = testing::NetworkArgs {
            network: testing::Network::DirectDial,
            orchestrator_port: None,
        };
        let plugin = testing::FakeNetworkPlugin::<u8>::from(&args);
        let lobby = testing::LobbyFakeNetworkPlugin::new(&plugin);
        build_plugin(&lobby, &mut app);
        let lobby_override = app
            .world()
            .get_resource::<discovery::LobbyTaskOverride>()
            .expect("idle lobby override");
        assert_eq!(lobby_override.orchestrator_port, None);
    }

    #[derive(Resource, Default, Clone, Deref, DerefMut)]
    struct ProofMap {
        entries: BTreeMap<u8, u32>,
    }

    impl room_sync::traits::ReplicatedState for ProofMap {
        type Op = u8;
        type Snapshot = Vec<(u8, u32)>;

        fn apply(&mut self, _meta: room_sync::OpMeta, op: &Self::Op) {
            self.entries.insert(*op, u32::from(*op));
        }

        fn snapshot(&self) -> Self::Snapshot {
            self.entries
                .iter()
                .map(|(key, value)| (*key, *value))
                .collect()
        }

        fn merge(&mut self, snapshot: Self::Snapshot) {
            for (key, value) in snapshot {
                self.entries.insert(key, value);
            }
        }

        fn slice_content_hashes(&self) -> room_sync::ReplicaSliceContentHashes {
            Vec::new()
        }

        fn slice_snapshot(&self, _index: room_sync::ReplicaSliceIndex) -> Self::Snapshot {
            self.snapshot()
        }
    }

    fn proof_app(game: &identity::GameId, network: &testing::NetworkArgs) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(discovery::FreenetPort(0));
        let fake = testing::FakeNetworkPlugin::<u32>::from(network);
        let lobby = testing::LobbyFakeNetworkPlugin::new(&fake);
        app.add_plugins(fake);
        app.add_plugins(lobby);
        app.add_plugins(plugin::P2PPlugin(
            plugin::Config::<u32>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
                .with_game(game.clone()),
        ));
        app.add_plugins(discovery::Plugin::new(discovery::Config {
            display_name: identity::Username::from("proof"),
            game: game.clone(),
            ..discovery::Config::default()
        }));
        app.add_plugins(room_sync::SyncPlugin::<ProofMap>::new(game.clone()));
        app.update();
        app
    }

    fn peer_id(app: &App) -> Option<identity::PeerId> {
        app.world()
            .get_resource::<p2p::ReadyChannel>()
            .and_then(|ready| ready.ready_rx.borrow().clone().map(|peer| peer.id))
    }

    fn state(app: &App) -> &discovery::state_machine::DiscoveryState {
        app.world()
            .resource::<discovery::state_machine::DiscoveryState>()
    }

    async fn wait_for(mut check: impl FnMut() -> bool) {
        let deadline = tokio::time::Instant::now()
            .checked_add(Duration::from_secs(20))
            .unwrap_or_else(tokio::time::Instant::now);
        loop {
            if check() {
                return;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "proof condition timed out"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn two_fake_peers_mesh_through_the_real_loops() {
        let orchestrator =
            testing::FakeNetworkServer::start_with(Box::new(testing::LobbyKit::default()))
                .await
                .expect("orchestrator");
        let network = testing::NetworkArgs {
            network: testing::Network::Fake,
            orchestrator_port: Some(*orchestrator),
        };
        let game = identity::GameId::from("proof-game");
        let other = identity::GameId::from("proof-other");
        let room = identity::RoomName::from("proof-room");
        let mut first = proof_app(&game, &network);
        let mut second = proof_app(&game, &network);
        let mut stranger = proof_app(&other, &network);
        for app in [&mut first, &mut second, &mut stranger] {
            let mut seen = false;
            for _ in 0..500 {
                app.update();
                if peer_id(app).is_some() {
                    seen = true;
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            assert!(seen, "fake peer never became ready");
        }
        first
            .world_mut()
            .resource_mut::<Messages<discovery::Command>>()
            .write(discovery::Command::Create(room.clone()));
        wait_for(|| {
            second.update();
            first.update();
            state(&second).lobby.contains_key(&room)
        })
        .await;
        second
            .world_mut()
            .resource_mut::<Messages<discovery::Command>>()
            .write(discovery::Command::Join(room.clone()));
        wait_for(|| {
            first.update();
            second.update();
            let a = state(&first);
            let b = state(&second);
            let a_ready = a.room.as_ref().is_some_and(|joined| {
                joined.phase == discovery::JoinPhase::Ready && a.connected_members().count() == 1
            });
            let b_ready = b.room.as_ref().is_some_and(|joined| {
                joined.phase == discovery::JoinPhase::Ready && b.connected_members().count() == 1
            });
            a_ready && b_ready
        })
        .await;
        for _ in 0..150 {
            stranger.update();
            assert!(
                !state(&stranger).lobby.contains_key(&room),
                "second game must not see the room"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let departed = state(&first).me.id.clone();
        first
            .world_mut()
            .resource_mut::<testing::FakeTaskHandles>()
            .abort_all();
        wait_for(|| {
            second.update();
            !second
                .world()
                .resource::<discovery::state_machine::DiscoveryState>()
                .connected_members()
                .any(|peer| peer == &departed)
        })
        .await;
    }
}
