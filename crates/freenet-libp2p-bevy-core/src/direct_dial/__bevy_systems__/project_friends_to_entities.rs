use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;

use crate::direct_dial;
use crate::identity;
use crate::peers;

pub fn project_friends_to_entities(
    mut commands: Commands,
    state: Option<Res<direct_dial::state_machine::DialState>>,
    found_peers: Query<(Entity, &peers::Peer)>,
    friends: Query<(Entity, &peers::Peer, Has<peers::KeepAlive>), With<direct_dial::Friend>>,
) {
    let Some(state) = state.map(Res::into_inner) else {
        return;
    };
    let friend_ids: BTreeSet<identity::PeerId> = state
        .entries
        .iter()
        .filter(|entry| entry.kind == direct_dial::KnownPeerKind::Friend)
        .map(|entry| entry.peer.id.clone())
        .collect();
    let mut peer_entity_by_id: BTreeMap<identity::PeerId, Entity> = BTreeMap::new();
    for (entity, peer) in &found_peers {
        peer_entity_by_id.insert((**peer).clone(), entity);
    }
    let mut current: BTreeMap<identity::PeerId, (Entity, bool)> = BTreeMap::new();
    for (entity, peer, has_keep_alive) in &friends {
        current.insert((**peer).clone(), (entity, has_keep_alive));
    }
    for id in &friend_ids {
        match current.get(id) {
            Some((_, true)) => {}
            Some((entity, false)) => {
                commands.entity(*entity).insert(peers::KeepAlive);
            }
            None => {
                let entity = peer_entity_by_id
                    .entry(id.clone())
                    .or_insert_with(|| commands.spawn(peers::Peer::from(id.clone())).id());
                let entity = *entity;
                commands.entity(entity).insert(direct_dial::Friend);
            }
        }
    }
    for (id, (entity, _)) in &current {
        if !friend_ids.contains(id) {
            commands.entity(*entity).remove::<direct_dial::Friend>();
            commands.entity(*entity).remove::<peers::KeepAlive>();
        }
    }
}
