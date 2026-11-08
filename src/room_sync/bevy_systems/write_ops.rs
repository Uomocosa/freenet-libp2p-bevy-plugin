use bevy::prelude::*;

use crate::identity;
use crate::p2p;
use crate::peers;
use crate::room_sync;

pub fn write_ops<Replica: room_sync::ReplicatedState>(
    mut writer: ResMut<room_sync::PendingOps<Replica>>,
    mut clock: ResMut<room_sync::LamportClock>,
    ready: Res<p2p::ReadyChannel>,
    me_rooms: Query<&peers::InRoom, With<peers::Me>>,
    room_names: Query<&peers::Room>,
    game: Res<room_sync::SyncGame>,
    mut room: ResMut<Replica>,
    mut pending: ResMut<p2p::PendingLibp2pCommands>,
) {
    let ready = ready.into_inner();
    let game = game.into_inner();
    let ops = writer.take_all();
    if ops.is_empty() {
        return;
    }
    let current = me_rooms
        .iter()
        .next()
        .and_then(|in_room| room_names.get(**in_room).ok())
        .map(|room| (**room).clone());
    let (Some(room_name), Some(me)) = (current, ready.ready_rx.borrow().clone()) else {
        writer.restore(ops);
        return;
    };
    let author = me.id.peer_author();
    let topic = identity::Topic::from_room_sync(game, &room_name);
    for op in ops {
        let meta = room_sync::OpMeta {
            lamport: clock.tick(),
            author,
        };
        room.apply(meta, &op);
        let message = room_sync::ReplicaSyncTopicMessage::ApplyThisOp {
            room: room_name.clone(),
            meta,
            op,
        };
        let Ok(data) = bincode::serialize(&message) else {
            continue;
        };
        pending.push(p2p::Libp2pCommand::Publish {
            topic: topic.clone(),
            data,
        });
    }
}
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bevy::prelude::*;
    use derive_more::{Deref, DerefMut};
    use serde::{Deserialize, Serialize};

    use crate::identity;
    use crate::p2p;
    use crate::peers;
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

    impl room_sync::ReplicatedState for Map {
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

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let game = identity::GameId::from("game");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin::P2PPlugin(
            plugin::Config::<u8>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
                .with_game(game.clone()),
        ));
        app.add_plugins(room_sync::SyncPlugin::<Map>::new(game));
        let room_entity = app
            .world_mut()
            .spawn(peers::Room::from(identity::RoomName::from("alpha")))
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
            peers::InRoom(room_entity),
        ));
        app.world_mut()
            .resource_mut::<room_sync::PendingOps<Map>>()
            .ops
            .push(Op { key: 1, value: 2 });
        app.update();

        assert_eq!(
            app.world().resource::<Map>().entries.get(&1).copied(),
            Some(2)
        );
    }
}
