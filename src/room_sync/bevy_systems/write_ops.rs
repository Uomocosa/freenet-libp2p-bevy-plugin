use bevy::prelude::*;

use crate::discovery;
use crate::p2p;
use crate::room_sync;

pub fn write_ops<S: room_sync::RoomState>(
    mut writer: ResMut<room_sync::RoomSync<S>>,
    mut clock: ResMut<room_sync::LamportClock>,
    ready: Res<p2p::ReadyChannel>,
    snapshot: Option<Res<discovery::Snapshot>>,
    game: Res<room_sync::RoomSyncGame>,
    mut pending: ResMut<p2p::PendingNetCommands>,
) {
    let ready = ready.into_inner();
    let snapshot = snapshot.map(Res::into_inner);
    let game = game.into_inner();
    let ops = writer.take_all();
    if ops.is_empty() {
        return;
    }
    let joined = snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.room.as_ref());
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
        let frame = room_sync::SyncFrame {
            room: joined.name.clone(),
            meta,
            op,
        };
        let Ok(data) = bincode::serialize(&frame) else {
            continue;
        };
        pending.push(p2p::NetCommand::Publish {
            topic: topic.clone(),
            data,
        });
    }
}

// no test_usage necessary
