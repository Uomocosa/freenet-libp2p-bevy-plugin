use bevy::prelude::*;

use crate::peers;

type OrphanPeerRow = (
    Entity,
    Has<peers::Connected>,
    Has<peers::InZone>,
    Has<peers::HeldZones>,
    Has<peers::KeepAlive>,
    Has<peers::Me>,
);

pub fn despawn_orphan_peers(
    mut commands: Commands,
    found_peers: Query<OrphanPeerRow, With<peers::Peer>>,
) {
    let orphans: Vec<Entity> = found_peers
        .iter()
        .filter(
            |(_, is_connected, is_in_zone, holds_history, is_keep_alive, is_me)| {
                !is_connected && !is_in_zone && !holds_history && !is_keep_alive && !is_me
            },
        )
        .map(|(entity, _, _, _, _, _)| entity)
        .collect();
    for entity in orphans {
        commands.entity(entity).despawn();
    }
}
