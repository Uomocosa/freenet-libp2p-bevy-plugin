use bevy::prelude::*;

use crate::discovery;
use crate::p2p;
use crate::room_sync;

pub fn write_ops<Replica: room_sync::RoomState>(
    mut writer: ResMut<room_sync::PendingOps<Replica>>,
    mut clock: ResMut<room_sync::LamportClock>,
    ready: Res<p2p::ReadyChannel>,
    status: Option<Res<discovery::Status>>,
    game: Res<room_sync::SyncGame>,
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
    let author = room_sync::peer_author(&me.id);
    let topic = room_sync::sync_topic(game, &joined.name);
    for op in ops {
        let meta = room_sync::OpMeta {
            lamport: clock.tick(),
            author,
        };
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

// no test_usage necessary
