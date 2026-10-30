use std::sync::Mutex;

use bevy::prelude::*;

use crate::discovery;
use crate::p2p;
use crate::room_sync;

pub fn build_plugin<Replica: room_sync::RoomState>(
    plugin: &room_sync::SyncPlugin<Replica>,
    app: &mut App,
) {
    let Some(net_events) = app
        .world()
        .get_resource::<p2p::NetEventChannel>()
        .map(p2p::NetEventChannel::subscribe)
    else {
        tracing::error!(target: "room_sync", "room_sync: p2p NetEventChannel missing");
        return;
    };
    app.init_resource::<Replica>();
    app.init_resource::<room_sync::LamportClock>();
    app.init_resource::<room_sync::PendingOps<Replica>>();
    app.init_resource::<room_sync::bevy_systems::SyncState<Replica::Op>>();
    app.add_message::<room_sync::ReplicaInputFromTimer<Replica::Op>>();
    if let Some(anti_entropy) = plugin.anti_entropy {
        app.insert_resource(room_sync::AntiEntropyTimer(Timer::new(
            anti_entropy.interval,
            TimerMode::Repeating,
        )));
        app.world_mut()
            .resource_mut::<room_sync::bevy_systems::SyncState<Replica::Op>>()
            .slice_anti_entropy
            .max_fetches_per_peer = anti_entropy.max_slice_fetches_per_peer;
        app.add_systems(
            Update,
            room_sync::bevy_systems::publish_slice_content_hashes::<Replica>
                .before(room_sync::bevy_systems::drive::<Replica>),
        );
    }
    if let Some(history) = plugin.history {
        app.insert_resource(history);
        app.insert_resource(room_sync::HistoryWriteTimer(Timer::new(
            history.write_interval,
            TimerMode::Repeating,
        )));
        {
            let mut sync_state = app
                .world_mut()
                .resource_mut::<room_sync::bevy_systems::SyncState<Replica::Op>>();
            sync_state.kademlia_history.is_enabled = true;
            sync_state.kademlia_history.max_fetch_retries = history.max_fetch_retries;
            sync_state.kademlia_history.per_room_bytes = history.per_room_bytes;
            sync_state.kademlia_history.record_ttl = history.record_ttl;
        }
        app.add_systems(
            Update,
            room_sync::bevy_systems::refresh_room_history::<Replica>
                .before(room_sync::bevy_systems::drive::<Replica>),
        );
    }
    app.insert_resource(room_sync::SyncGame(plugin.game.clone()));
    app.insert_resource(room_sync::bevy_systems::NetEvents {
        receiver: Mutex::new(net_events),
    });
    app.add_systems(
        Update,
        (
            room_sync::bevy_systems::drive::<Replica>
                .after(discovery::bevy_systems::drain_events)
                .after(discovery::bevy_systems::drain_status),
            room_sync::bevy_systems::write_ops::<Replica>,
        ),
    );
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bevy::prelude::*;
    use derive_more::{Deref, DerefMut};
    use serde::{Deserialize, Serialize};

    use crate::identity;
    use crate::p2p;
    use crate::plugin;
    use crate::room_sync;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    struct Op {
        key: u8,
        value: u32,
    }

    #[derive(Resource, Default, Deref, DerefMut)]
    struct Map {
        entries: BTreeMap<u8, u32>,
    }

    impl room_sync::RoomState for Map {
        type Op = Op;
        type Snapshot = Vec<(u8, u32)>;

        fn apply(&mut self, _meta: room_sync::OpMeta, op: &Self::Op) {
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

        fn slice_content_hashes(&self) -> room_sync::ReplicaSliceContentHashes {
            vec![(
                room_sync::ReplicaSliceIndex(0),
                room_sync::ReplicaSliceContentHash(u64::try_from(self.entries.len()).unwrap_or(0)),
            )]
        }

        fn slice_snapshot(&self, _index: room_sync::ReplicaSliceIndex) -> Self::Snapshot {
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
            room_sync::SyncPlugin::<Map>::new(identity::GameId::from("test"))
                .with_history_config(Some(room_sync::HistoryConfig::default())),
        );
        app.update();
        assert!(
            app.world()
                .get_resource::<room_sync::PendingOps<Map>>()
                .is_some()
        );
        assert!(app.world().get_resource::<Map>().is_some());
        assert!(
            app.world()
                .get_resource::<room_sync::HistoryConfig>()
                .is_some()
        );
    }
}
