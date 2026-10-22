use bevy::prelude::*;

use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::room_sync;

pub fn write_ops<Replica: room_sync::RoomState>(
    mut writer: ResMut<room_sync::PendingOps<Replica>>,
    mut clock: ResMut<room_sync::LamportClock>,
    ready: Res<p2p::ReadyChannel>,
    status: Option<Res<discovery::Status>>,
    game: Res<room_sync::SyncGame>,
    mut room: ResMut<Replica>,
    mut pending: ResMut<p2p::PendingNetCommands>,
) {
    let ready = ready.into_inner();
    let status = status.map(Res::into_inner);
    let game = game.into_inner();
    let ops = writer.take_all();
    if ops.is_empty() {
        return;
    }
    let joined = status.as_ref().and_then(|status| status.room.as_ref());
    let (Some(joined), Some(me)) = (joined, ready.ready_rx.borrow().clone()) else {
        writer.restore(ops);
        return;
    };
    let author = me.id.peer_author();
    let topic = identity::Topic::for_room_sync(game, &joined.name);
    for op in ops {
        let meta = room_sync::OpMeta {
            lamport: clock.tick(),
            author,
        };
        room.apply(meta, &op);
        let message = room_sync::ReplicaSyncTopicMessage::ApplyThisOp {
            room: joined.name.clone(),
            meta,
            op,
        };
        let Ok(data) = bincode::serialize(&message) else {
            continue;
        };
        pending.push(p2p::NetCommand::Publish {
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

    use crate::discovery;
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

    fn joined_status() -> discovery::Status {
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(0),
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
            }),
            ..discovery::Status::default()
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
        app.add_message::<discovery::Event>();
        app.insert_resource(joined_status());
        let _ = app
            .world()
            .resource::<p2p::ReadyChannel>()
            .ready_tx
            .send(Some(identity::Peer {
                id: identity::PeerId::from("me"),
                addrs: Vec::new(),
            }));
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
