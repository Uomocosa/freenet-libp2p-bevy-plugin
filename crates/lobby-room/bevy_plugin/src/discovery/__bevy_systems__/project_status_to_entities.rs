use std::collections::{BTreeMap, BTreeSet, HashMap};

use bevy::prelude::*;

use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::peers;
use crate::replica_sync;
use discovery::state_machine::DiscoveryState;

type RoomRow<'a> = (
    Entity,
    &'a identity::ZoneName,
    Option<&'a p2p::HistoryKeyPrefix>,
    Option<&'a replica_sync::state_machine::ZoneLiveness>,
);

type PeerRow<'a> = (Entity, &'a peers::Peer, Option<&'a peers::InZone>);

type HoldLinkRow<'a> = (Entity, &'a peers::HeldBy, &'a peers::HoldOf);

pub fn project_status_to_entities(
    mut commands: Commands,
    state: Res<DiscoveryState>,
    game: Option<Res<replica_sync::SyncGame>>,
    room_entities: Query<RoomRow<'_>>,
    peer_entities: Query<PeerRow<'_>>,
    links: Query<HoldLinkRow<'_>, With<peers::HistoryHold>>,
) {
    let state = state.into_inner();
    let Some(game) = game.map(Res::into_inner) else {
        return;
    };
    let mut room_entity_by_name: BTreeMap<discovery::RoomName, Entity> = BTreeMap::new();
    for (entity, room, _, _) in &room_entities {
        room_entity_by_name.insert(discovery::RoomName::from(room.clone()), entity);
    }
    for room_name in DiscoveryState::desired_rooms(state) {
        let room_entity = room_entity_by_name
            .entry(room_name.clone())
            .or_insert_with(|| commands.spawn(identity::ZoneName::clone(&room_name)).id());
        let room_entity = *room_entity;
        let (prefix, liveness) =
            DiscoveryState::room_prefix_and_lobby_status(Some(state), game, &room_name);
        let is_room_in_sync =
            room_entities
                .get(room_entity)
                .is_ok_and(|(_, _, current_prefix, current_liveness)| {
                    current_prefix == Some(&prefix) && current_liveness == Some(&liveness)
                });
        if !is_room_in_sync {
            commands.entity(room_entity).insert((prefix, liveness));
        }
    }
    let (desired_in_room, desired_holds) =
        DiscoveryState::desired_membership(state, &room_entity_by_name);
    let mut peer_entity_by_id: BTreeMap<identity::PeerId, Entity> = BTreeMap::new();
    for (entity, peer, _) in &peer_entities {
        peer_entity_by_id.insert((**peer).clone(), entity);
    }
    let mut current_in_room: BTreeMap<identity::PeerId, BTreeSet<Entity>> = BTreeMap::new();
    let mut current_holds: BTreeMap<identity::PeerId, BTreeSet<Entity>> = BTreeMap::new();
    let mut peer_id_by_entity: HashMap<Entity, identity::PeerId> = HashMap::new();
    for (entity, peer, in_room) in &peer_entities {
        let id = (**peer).clone();
        peer_id_by_entity.insert(entity, id.clone());
        if let Some(target) = in_room {
            current_in_room.entry(id).or_default().insert(**target);
        }
    }
    let mut link_by_hold: BTreeMap<(identity::PeerId, Entity), Entity> = BTreeMap::new();
    for (link, held_by, hold_of) in &links {
        let Some(id) = peer_id_by_entity.get(&**held_by) else {
            continue;
        };
        link_by_hold.insert((id.clone(), **hold_of), link);
        current_holds
            .entry(id.clone())
            .or_default()
            .insert(**hold_of);
    }
    let mut peers_to_visit: BTreeSet<identity::PeerId> = BTreeSet::new();
    peers_to_visit.extend(desired_in_room.keys().cloned());
    peers_to_visit.extend(desired_holds.keys().cloned());
    peers_to_visit.extend(current_in_room.keys().cloned());
    peers_to_visit.extend(current_holds.keys().cloned());
    for peer in &peers_to_visit {
        let entity = peer_entity_by_id
            .entry(peer.clone())
            .or_insert_with(|| commands.spawn(peers::Peer::from(peer.clone())).id());
        let entity = *entity;
        let wants_room = desired_in_room.get(peer).copied();
        let is_in_room_in_sync = wants_room.map_or_else(
            || current_in_room.get(peer).is_none_or(BTreeSet::is_empty),
            |room| {
                current_in_room
                    .get(peer)
                    .is_some_and(|rooms| rooms.len() == 1 && rooms.contains(&room))
            },
        );
        if !is_in_room_in_sync {
            commands.entity(entity).remove::<peers::InZone>();
            if let Some(room) = wants_room {
                commands.entity(entity).insert(peers::InZone(room));
            }
        }
        let wants_holds = desired_holds.get(peer).cloned().unwrap_or_default();
        let has_holds = current_holds.get(peer).cloned().unwrap_or_default();
        for room in wants_holds.difference(&has_holds) {
            commands.spawn((
                peers::HistoryHold,
                peers::HeldBy(entity),
                peers::HoldOf(*room),
            ));
        }
        for room in has_holds.difference(&wants_holds) {
            if let Some(link) = link_by_hold.get(&(peer.clone(), *room)).copied() {
                commands.entity(link).despawn();
            }
        }
    }
}
