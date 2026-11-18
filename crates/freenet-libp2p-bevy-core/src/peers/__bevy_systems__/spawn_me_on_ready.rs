use bevy::prelude::*;

use crate::p2p;
use crate::peers;

pub fn spawn_me_on_ready(
    mut commands: Commands,
    ready: Option<Res<p2p::ReadyChannel>>,
    me_entities: Query<Entity, With<peers::Me>>,
    found_peers: Query<(Entity, &peers::Peer)>,
) {
    let snapshot = ready.and_then(|ready| ready.ready_rx.borrow().clone());
    let Some(me) = snapshot else {
        return;
    };
    if !me_entities.is_empty() {
        return;
    }
    let entity = found_peers
        .iter()
        .find(|(_, peer)| **peer == me.id)
        .map_or_else(
            || commands.spawn(peers::Peer::from(me.id.clone())).id(),
            |(entity, _)| entity,
        );
    commands.entity(entity).insert(peers::Me);
}
