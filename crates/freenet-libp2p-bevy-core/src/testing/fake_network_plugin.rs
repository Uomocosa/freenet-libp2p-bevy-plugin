use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::{App, Plugin};

use crate::identity;
use crate::p2p;
use crate::testing;

static PEER_COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct FakeNetworkPlugin<Payload> {
    pub network: testing::Network,
    pub orchestrator_port: Option<u16>,
    pub peer: identity::Peer,
    pub(crate) marker: PhantomData<Payload>,
}

impl<Payload> From<&testing::NetworkArgs> for FakeNetworkPlugin<Payload> {
    fn from(args: &testing::NetworkArgs) -> Self {
        let index = PEER_COUNTER.fetch_add(1, Ordering::Relaxed);
        let process = std::process::id();
        let peer = identity::Peer {
            id: identity::PeerId::from(format!("fake-peer-{process}-{index}")),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        };
        Self {
            network: args.network,
            orchestrator_port: args.orchestrator_port,
            peer,
            marker: PhantomData,
        }
    }
}

#[rustfmt::skip]
impl<Payload: p2p::traits::Message> Plugin for FakeNetworkPlugin<Payload> {
    fn build(&self, app: &mut App) { self.build_plugin(app); }
}

#[atomic_delegates]
impl<Payload: p2p::traits::Message> FakeNetworkPlugin<Payload> {
    pub fn build_plugin(&self, app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use bevy::prelude::*;
    use derive_more::{Deref, DerefMut};

    use super::FakeNetworkPlugin;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::plugin;
    use crate::room_sync;
    use crate::testing;

    #[test]
    fn test_usage() {
        let args = testing::NetworkArgs {
            network: testing::Network::Mainnet,
            orchestrator_port: None,
        };
        let plugin = FakeNetworkPlugin::<u8>::from(&args);
        assert!(matches!(plugin.network, testing::Network::Mainnet));
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
        app.add_plugins(FakeNetworkPlugin::<u32>::from(network));
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
        let orchestrator = testing::FakeNetworkServer::start()
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
