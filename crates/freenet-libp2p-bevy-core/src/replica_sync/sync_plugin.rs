use std::marker::PhantomData;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::*;

use crate::identity;
use crate::p2p;
use crate::replica_sync;
use std::sync::Mutex;

pub struct SyncPlugin<Replica: replica_sync::traits::ReplicatedState> {
    pub game: identity::GameId,
    pub anti_entropy: Option<replica_sync::AntiEntropyConfig>,
    pub history: Option<replica_sync::HistoryConfig>,
    pub marker: PhantomData<Replica>,
}

impl<Replica: replica_sync::traits::ReplicatedState> SyncPlugin<Replica> {
    #[must_use]
    pub fn new(game: identity::GameId) -> Self {
        Self {
            game,
            anti_entropy: Some(replica_sync::AntiEntropyConfig::default()),
            history: None,
            marker: PhantomData,
        }
    }
}

#[rustfmt::skip]
impl<Replica: replica_sync::traits::ReplicatedState> bevy::prelude::Plugin for SyncPlugin<Replica> {
    fn build(&self, app: &mut App) {
        let Some(libp2p_events) = app
            .world()
            .get_resource::<p2p::Libp2pEventChannel>()
            .map(p2p::Libp2pEventChannel::subscribe)
        else {
            tracing::error!(target: "replica_sync", "replica_sync: p2p Libp2pEventChannel missing");
            return;
        };
        app.init_resource::<Replica>();
        app.init_resource::<replica_sync::LamportClock>();
        app.init_resource::<replica_sync::PendingOps<Replica>>();
        app.init_resource::<replica_sync::bevy_systems::SyncState<Replica::Op>>();
        app.add_message::<replica_sync::ReplicaInputFromTimer<Replica::Op>>();
        app.add_message::<replica_sync::JoinFailure>();
        if let Some(anti_entropy) = self.anti_entropy {
            app.insert_resource(replica_sync::AntiEntropyTimer(Timer::new(
                anti_entropy.interval,
                TimerMode::Repeating,
            )));
            app.world_mut()
                .resource_mut::<replica_sync::bevy_systems::SyncState<Replica::Op>>()
                .slice_anti_entropy
                .max_fetches_per_peer = anti_entropy.max_slice_fetches_per_peer;
            app.add_systems(
                Update,
                replica_sync::bevy_systems::publish_slice_content_hashes::<Replica>
                    .before(replica_sync::bevy_systems::drive::<Replica>),
            );
        }
        if let Some(history) = self.history {
            app.insert_resource(history);
            app.insert_resource(replica_sync::HistoryWriteTimer(Timer::new(
                history.write_interval,
                TimerMode::Repeating,
            )));
            {
                let mut sync_state = app
                    .world_mut()
                    .resource_mut::<replica_sync::bevy_systems::SyncState<Replica::Op>>();
                sync_state.kademlia_history.is_enabled = true;
                sync_state.kademlia_history.max_fetch_retries = history.max_fetch_retries;
                sync_state.kademlia_history.per_zone_bytes = history.per_zone_bytes;
                sync_state.kademlia_history.record_ttl = history.record_ttl;
            }
            app.add_systems(
                Update,
                replica_sync::bevy_systems::refresh_zone_history::<Replica>
                    .before(replica_sync::bevy_systems::drive::<Replica>),
            );
        }
        app.insert_resource(replica_sync::SyncGame(self.game.clone()));
        app.insert_resource(replica_sync::bevy_systems::Libp2pEvents {
            receiver: Mutex::new(libp2p_events),
        });
        app.configure_sets(
            Update,
            replica_sync::ReplicaSyncSet::Input.before(replica_sync::ReplicaSyncSet::Drive),
        );
        app.add_systems(
            Update,
            (
                replica_sync::bevy_systems::drive::<Replica>.in_set(replica_sync::ReplicaSyncSet::Drive),
                replica_sync::bevy_systems::write_ops::<Replica>,
            ),
        );
    }
}

impl<Replica: replica_sync::traits::ReplicatedState> SyncPlugin<Replica> {
    #[must_use]
    pub const fn with_anti_entropy_config(
        mut self,
        anti_entropy: Option<replica_sync::AntiEntropyConfig>,
    ) -> Self {
        self.anti_entropy = anti_entropy;
        self
    }

    #[must_use]
    pub const fn with_history_config(
        mut self,
        history: Option<replica_sync::HistoryConfig>,
    ) -> Self {
        self.history = history;
        self
    }
}

#[atomic_delegates]
impl<Replica: replica_sync::traits::ReplicatedState> SyncPlugin<Replica> {}

// no test_usage necessary

#[cfg(test)]
mod build_plugin_tests {
    use std::collections::BTreeMap;

    use bevy::prelude::*;
    use derive_more::{Deref, DerefMut};
    use serde::{Deserialize, Serialize};

    use crate::identity;
    use crate::p2p;
    use crate::peers;
    use crate::plugin;
    use crate::replica_sync;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    struct Op {
        key: u8,
        value: u32,
    }

    #[derive(Resource, Default, Deref, DerefMut)]
    struct Map {
        entries: BTreeMap<u8, u32>,
    }

    impl replica_sync::traits::ReplicatedState for Map {
        type Op = Op;
        type Snapshot = Vec<(u8, u32)>;

        fn apply(&mut self, _meta: replica_sync::OpMeta, op: &Self::Op) {
            self.entries.insert(op.key, op.value);
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

        fn slice_content_hashes(&self) -> replica_sync::ReplicaSliceContentHashes {
            vec![(
                replica_sync::ReplicaSliceIndex(0),
                replica_sync::ReplicaSliceContentHash(
                    u64::try_from(self.entries.len()).unwrap_or(0),
                ),
            )]
        }

        fn slice_snapshot(&self, _index: replica_sync::ReplicaSliceIndex) -> Self::Snapshot {
            self.snapshot()
        }
    }

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin::P2PPlugin(plugin::Config::<u8>::new(
            p2p::TransportMode::Both,
            p2p::MdnsMode::Disabled,
        )));
        app.add_plugins(
            replica_sync::SyncPlugin::<Map>::new(identity::GameId::from("test"))
                .with_history_config(Some(replica_sync::HistoryConfig::default())),
        );
        app.update();
        assert!(
            app.world()
                .get_resource::<replica_sync::PendingOps<Map>>()
                .is_some()
        );
        assert!(app.world().get_resource::<Map>().is_some());
        assert!(
            app.world()
                .get_resource::<replica_sync::HistoryConfig>()
                .is_some()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn write_ops_applies_pending_ops_to_the_replica() {
        let game = identity::GameId::from("game");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin::P2PPlugin(
            plugin::Config::<u8>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
                .with_game(game.clone()),
        ));
        app.add_plugins(replica_sync::SyncPlugin::<Map>::new(game));
        let zone_entity = app
            .world_mut()
            .spawn(identity::ZoneName::from("alpha"))
            .id();
        let _ = app
            .world()
            .resource::<p2p::ReadyChannel>()
            .ready_tx
            .send(Some(identity::Peer {
                id: identity::PeerId::from("me"),
                addrs: Vec::new(),
            }));
        app.world_mut().spawn((
            peers::Peer::from(identity::PeerId::from("me")),
            peers::Me,
            peers::InZone(zone_entity),
        ));
        app.world_mut()
            .resource_mut::<replica_sync::PendingOps<Map>>()
            .ops
            .push(Op { key: 1, value: 2 });
        app.update();

        assert_eq!(
            app.world().resource::<Map>().entries.get(&1).copied(),
            Some(2)
        );
    }
}
