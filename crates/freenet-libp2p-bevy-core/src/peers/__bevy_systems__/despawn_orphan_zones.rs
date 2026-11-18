use std::collections::HashSet;

use bevy::prelude::*;

use crate::identity;
use crate::peers;

pub fn despawn_orphan_zones(
    mut commands: Commands,
    found_zones: Query<(
        Entity,
        &identity::ZoneName,
        Option<&peers::ZoneMembers>,
        Option<&peers::HistoryHolders>,
    )>,
    me_zones: Query<&peers::InZone, With<peers::Me>>,
) {
    let held_by_me: HashSet<Entity> = me_zones.iter().map(|in_zone| **in_zone).collect();
    let orphans: Vec<Entity> = found_zones
        .iter()
        .filter(|(entity, _, members, holders)| {
            members.is_none() && holders.is_none() && !held_by_me.contains(entity)
        })
        .map(|(entity, _, _, _)| entity)
        .collect();
    for entity in orphans {
        commands.entity(entity).despawn();
    }
}
