use std::collections::HashMap;

use bevy::prelude::*;

use crate::identity;
use crate::p2p;
use crate::peers;

pub fn track_peer_connections(
    mut commands: Commands,
    mut tap: Option<ResMut<peers::PeerEventTap>>,
    found_peers: Query<(Entity, &peers::Peer)>,
) {
    let Some(tap) = tap.as_deref_mut() else {
        return;
    };
    let mut peer_entity_by_id: HashMap<identity::PeerId, Entity> = HashMap::new();
    for (entity, peer) in &found_peers {
        peer_entity_by_id.insert((**peer).clone(), entity);
    }
    while let Ok(event) = tap.try_recv() {
        match event {
            p2p::Libp2pEvent::PeerConnected(id)
            | p2p::Libp2pEvent::Libp2pConnectionEstablished { peer_id: id, .. } => {
                let entity = peer_entity_by_id
                    .entry(id.clone())
                    .or_insert_with(|| commands.spawn(peers::Peer::from(id.clone())).id());
                let entity = *entity;
                commands.entity(entity).insert(peers::Connected);
            }
            p2p::Libp2pEvent::PeerDisconnected(id) => {
                if let Some(entity) = peer_entity_by_id.get(&id).copied() {
                    commands.entity(entity).remove::<peers::Connected>();
                }
            }
            _ => {}
        }
    }
}
