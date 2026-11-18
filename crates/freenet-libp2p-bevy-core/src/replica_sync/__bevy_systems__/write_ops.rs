use bevy::prelude::*;

use crate::identity;
use crate::p2p;
use crate::peers;
use crate::replica_sync;

pub fn write_ops<Replica: replica_sync::traits::ReplicatedState>(
    mut writer: ResMut<replica_sync::PendingOps<Replica>>,
    mut clock: ResMut<replica_sync::LamportClock>,
    ready: Res<p2p::ReadyChannel>,
    me_zones: Query<&peers::InZone, With<peers::Me>>,
    zone_names: Query<&identity::ZoneName>,
    game: Res<replica_sync::SyncGame>,
    mut zone: ResMut<Replica>,
    mut commands: MessageWriter<p2p::Libp2pCommand>,
) {
    let ready = ready.into_inner();
    let game = game.into_inner();
    let ops = writer.take_all();
    if ops.is_empty() {
        return;
    }
    let current = me_zones
        .iter()
        .next()
        .and_then(|in_zone| zone_names.get(**in_zone).ok())
        .cloned();
    let (Some(zone_name), Some(me)) = (current, ready.ready_rx.borrow().clone()) else {
        writer.restore(ops);
        return;
    };
    let author = me.id.peer_author();
    let topic = identity::Topic::from_replica_sync(game, &zone_name);
    for op in ops {
        let meta = replica_sync::OpMeta {
            lamport: clock.tick(),
            author,
        };
        zone.apply(meta, &op);
        let message = replica_sync::ReplicaSyncTopicMessage::ApplyThisOp {
            zone: zone_name.clone(),
            meta,
            op,
        };
        let Ok(data) = bincode::serialize(&message) else {
            continue;
        };
        commands.write(p2p::Libp2pCommand::Publish {
            topic: topic.clone(),
            data,
        });
    }
}
