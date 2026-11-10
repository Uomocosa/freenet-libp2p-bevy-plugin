use std::collections::{BTreeMap, BTreeSet, HashMap};

use bevy::prelude::*;

use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::peers;
use crate::room_sync;
use discovery::state_machine::DiscoveryState;

type RoomRow<'a> = (
    Entity,
    &'a peers::Room,
    Option<&'a p2p::HistoryKeyPrefix>,
    Option<&'a room_sync::state_machine::RoomLiveness>,
);

type PeerRow<'a> = (Entity, &'a peers::Peer, Option<&'a peers::InRoom>);

type HoldLinkRow<'a> = (Entity, &'a peers::HeldBy, &'a peers::HoldOf);

pub fn project_status_to_entities(
    mut commands: Commands,
    state: Res<DiscoveryState>,
    game: Option<Res<room_sync::SyncGame>>,
    room_entities: Query<RoomRow<'_>>,
    peer_entities: Query<PeerRow<'_>>,
    links: Query<HoldLinkRow<'_>, With<peers::HistoryHold>>,
) {
    let state = state.into_inner();
    let Some(game) = game.map(Res::into_inner) else {
        return;
    };
    let mut room_entity_by_name: BTreeMap<identity::RoomName, Entity> = BTreeMap::new();
    for (entity, room, _, _) in &room_entities {
        room_entity_by_name.insert((**room).clone(), entity);
    }
    for room_name in DiscoveryState::desired_rooms(state) {
        let room_entity = room_entity_by_name
            .entry(room_name.clone())
            .or_insert_with(|| commands.spawn(peers::Room::from(room_name.clone())).id());
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
            commands.entity(entity).remove::<peers::InRoom>();
            if let Some(room) = wants_room {
                commands.entity(entity).insert(peers::InRoom(room));
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bevy::prelude::*;

    use super::project_status_to_entities;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::peers;
    use crate::room_sync;
    use crate::testing;
    use crate::time;
    use discovery::state_machine::DiscoveryState;

    fn member_with(
        id: &str,
        membership: discovery::RoomMembership,
    ) -> (identity::PeerId, discovery::RoomMember) {
        let peer_id = identity::PeerId::from(id);
        (
            peer_id.clone(),
            discovery::RoomMember {
                peer: identity::Peer {
                    id: peer_id,
                    addrs: Vec::new(),
                },
                updated_at: time::UnixTime::from_secs(10),
                display_name: None,
                membership,
            },
        )
    }

    fn occupant(id: &str) -> (identity::PeerId, discovery::RoomMember) {
        member_with(id, discovery::RoomMembership::Occupant)
    }

    fn holder(id: &str) -> (identity::PeerId, discovery::RoomMember) {
        member_with(id, discovery::RoomMembership::HistoryHolder)
    }

    fn lobby_with_room(
        room: &identity::RoomName,
        members: Vec<(identity::PeerId, discovery::RoomMember)>,
    ) -> discovery::Lobby {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room.clone(),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::from_secs(10),
                members: members.into_iter().collect(),
            },
        );
        lobby
    }

    fn state_with_lobby(lobby: discovery::Lobby) -> DiscoveryState {
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build()
    }

    fn app_with_projection(state: DiscoveryState, game: identity::GameId) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(state);
        app.insert_resource(room_sync::SyncGame(game));
        app.add_systems(Update, project_status_to_entities);
        app
    }

    fn room_entity(app: &mut App, room: &identity::RoomName) -> Entity {
        let found = room_by_name(app, room);
        assert_eq!(found.len(), 1);
        found[0]
    }

    fn peer_entity(app: &mut App, id: &str) -> Entity {
        let found = peer_by_id(app, &identity::PeerId::from(id));
        assert_eq!(found.len(), 1);
        found[0]
    }

    fn hold_link_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<peers::HistoryHold>>()
            .iter(app.world())
            .count()
    }

    fn room_by_name(app: &mut App, name: &identity::RoomName) -> Vec<Entity> {
        let mut found = app.world_mut().query::<(Entity, &peers::Room)>();
        found
            .iter(app.world())
            .filter(|(_, room)| ***room == *name)
            .map(|(entity, _)| entity)
            .collect()
    }

    fn peer_by_id(app: &mut App, id: &identity::PeerId) -> Vec<Entity> {
        let mut found = app.world_mut().query::<(Entity, &peers::Peer)>();
        found
            .iter(app.world())
            .filter(|(_, peer)| ***peer == *id)
            .map(|(entity, _)| entity)
            .collect()
    }

    #[test]
    fn stale_room_entity_is_not_reused() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-a");
        let state = state_with_lobby(lobby_with_room(&room, vec![occupant("alice")]));
        let mut app = app_with_projection(state, game);
        app.update();
        let old = room_by_name(&mut app, &room);
        assert_eq!(old.len(), 1);
        app.world_mut().despawn(old[0]);
        app.update();
        let fresh = room_by_name(&mut app, &room);
        assert_eq!(fresh.len(), 1);
        assert_ne!(fresh[0], old[0]);
        assert!(app.world().get_entity(fresh[0]).is_ok());
    }

    #[test]
    fn stale_peer_entity_is_not_reused() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-a");
        let state = state_with_lobby(lobby_with_room(&room, vec![occupant("alice")]));
        let mut app = app_with_projection(state, game);
        app.update();
        let old = peer_by_id(&mut app, &identity::PeerId::from("alice"));
        assert_eq!(old.len(), 1);
        app.world_mut().despawn(old[0]);
        app.update();
        let fresh = peer_by_id(&mut app, &identity::PeerId::from("alice"));
        assert_eq!(fresh.len(), 1);
        assert_ne!(fresh[0], old[0]);
        assert!(app.world().get_entity(fresh[0]).is_ok());
    }

    #[test]
    fn holder_of_two_rooms_links_both_without_churn() {
        let game = identity::GameId::from("game");
        let room_a = identity::RoomName::from("room-a");
        let room_b = identity::RoomName::from("room-b");
        let mut lobby = lobby_with_room(&room_a, vec![holder("alice")]);
        lobby.insert(
            room_b.clone(),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::from_secs(10),
                members: BTreeMap::from([holder("alice")]),
            },
        );
        let state = state_with_lobby(lobby);
        let mut app = app_with_projection(state, game);
        app.update();
        assert_eq!(hold_link_count(&mut app), 2);
        let joined = room_entity(&mut app, &room_a);
        let other = room_entity(&mut app, &room_b);
        let alice = peer_entity(&mut app, "alice");
        let held_rooms = app
            .world()
            .get::<peers::HeldRooms>(alice)
            .expect("held rooms");
        assert_eq!(held_rooms.len(), 2);
        for room in [joined, other] {
            assert!(
                app.world()
                    .get::<peers::HistoryHolders>(room)
                    .is_some_and(|holders| holders.len() == 1)
            );
        }
        for _ in 0..4 {
            app.update();
            assert_eq!(hold_link_count(&mut app), 2);
        }
    }

    #[test]
    fn departed_holder_frees_its_links_and_the_room_is_swept() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-a");
        let state = state_with_lobby(lobby_with_room(&room, vec![holder("alice")]));
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(state);
        app.insert_resource(room_sync::SyncGame(game));
        app.add_systems(
            Update,
            (
                project_status_to_entities,
                peers::bevy_systems::despawn_orphan_rooms,
            )
                .chain(),
        );
        app.update();
        assert_eq!(hold_link_count(&mut app), 1);
        let room_entity = room_entity(&mut app, &room);
        assert!(app.world().get_entity(room_entity).is_ok());
        let alice = identity::PeerId::from("alice");
        let mut state = app.world_mut().resource_mut::<DiscoveryState>();
        state
            .lobby
            .get_mut(&room)
            .expect("room in lobby")
            .members
            .get_mut(&alice)
            .expect("alice listed")
            .membership = discovery::RoomMembership::Departed;
        state.lobby.remove(&room);
        app.update();
        app.update();
        assert_eq!(hold_link_count(&mut app), 0);
        assert!(app.world().get_entity(room_entity).is_err());
        assert_eq!(room_by_name(&mut app, &room).len(), 0);
    }

    #[test]
    fn test_usage() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-a");
        let state = state_with_lobby(lobby_with_room(
            &room,
            vec![occupant("alice"), occupant("bob")],
        ));
        let (expected_prefix, expected_liveness) =
            DiscoveryState::room_prefix_and_lobby_status(Some(&state), &game, &room);
        let mut app = app_with_projection(state, game);
        app.update();
        let room_entity = room_entity(&mut app, &room);
        assert_eq!(
            app.world().get::<p2p::HistoryKeyPrefix>(room_entity),
            Some(&expected_prefix)
        );
        assert_eq!(
            app.world()
                .get::<room_sync::state_machine::RoomLiveness>(room_entity),
            Some(&expected_liveness)
        );
        for name in ["alice", "bob"] {
            let peer = peer_entity(&mut app, name);
            assert_eq!(
                app.world()
                    .get::<peers::InRoom>(peer)
                    .map(|in_room| **in_room),
                Some(room_entity)
            );
        }
        let members = app
            .world()
            .get::<peers::RoomMembers>(room_entity)
            .expect("members filled");
        assert_eq!(members.len(), 2);
    }

    #[test]
    fn member_who_departed_loses_in_room() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-a");
        let state = state_with_lobby(lobby_with_room(
            &room,
            vec![occupant("alice"), occupant("bob")],
        ));
        let mut app = app_with_projection(state, game);
        app.update();
        let alice = identity::PeerId::from("alice");
        app.world_mut()
            .resource_mut::<DiscoveryState>()
            .lobby
            .get_mut(&room)
            .expect("room in lobby")
            .members
            .get_mut(&alice)
            .expect("alice listed")
            .membership = discovery::RoomMembership::Departed;
        app.update();
        let alice_entity = peer_entity(&mut app, "alice");
        assert!(app.world().get::<peers::InRoom>(alice_entity).is_none());
        let room_entity = room_entity(&mut app, &room);
        let members = app
            .world()
            .get::<peers::RoomMembers>(room_entity)
            .expect("members filled");
        assert!(!members.contains(&alice_entity));
        let bob_entity = peer_entity(&mut app, "bob");
        assert_eq!(
            app.world()
                .get::<peers::InRoom>(bob_entity)
                .map(|in_room| **in_room),
            Some(room_entity)
        );
    }

    #[test]
    fn me_gains_and_loses_in_room_with_join_and_leave() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-a");
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .in_room(discovery::JoinedRoom {
                name: room.clone(),
                created_at: time::UnixTime::from_secs(10),
                phase: discovery::JoinPhase::Ready,
                members: BTreeMap::from([occupant("me")]),
                how_entered: discovery::HowRoomWasEntered::Created,
            })
            .build();
        let mut app = app_with_projection(state, game);
        app.update();
        let room_entity = room_entity(&mut app, &room);
        let me_entity = peer_entity(&mut app, "me");
        assert_eq!(
            app.world()
                .get::<peers::InRoom>(me_entity)
                .map(|in_room| **in_room),
            Some(room_entity)
        );
        app.world_mut().resource_mut::<DiscoveryState>().room = None;
        app.update();
        assert!(app.world().get::<peers::InRoom>(me_entity).is_none());
    }

    #[test]
    fn peer_listed_in_two_rooms_keeps_the_joined_room() {
        let game = identity::GameId::from("game");
        let room_a = identity::RoomName::from("room-a");
        let room_b = identity::RoomName::from("room-b");
        let mut lobby = lobby_with_room(&room_a, vec![occupant("alice")]);
        lobby.insert(
            room_b.clone(),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::from_secs(10),
                members: BTreeMap::from([occupant("alice"), occupant("bob")]),
            },
        );
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .in_room(discovery::JoinedRoom {
                name: room_a.clone(),
                created_at: time::UnixTime::from_secs(10),
                phase: discovery::JoinPhase::Ready,
                members: BTreeMap::from([occupant("alice")]),
                how_entered: discovery::HowRoomWasEntered::Joined,
            })
            .build();
        let mut app = app_with_projection(state, game);
        app.update();
        let joined = room_entity(&mut app, &room_a);
        let other = room_entity(&mut app, &room_b);
        let alice_entity = peer_entity(&mut app, "alice");
        assert_eq!(
            app.world()
                .get::<peers::InRoom>(alice_entity)
                .map(|in_room| **in_room),
            Some(joined)
        );
        let bob_entity = peer_entity(&mut app, "bob");
        assert_eq!(
            app.world()
                .get::<peers::InRoom>(bob_entity)
                .map(|in_room| **in_room),
            Some(other)
        );
    }

    #[test]
    fn discovery_plugin_registers_the_projection() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<AppExit>();
        app.add_plugins(discovery::Plugin::new(discovery::Config::default()));
        let game = identity::GameId::from("test");
        app.insert_resource(room_sync::SyncGame(game.clone()));
        let room = identity::RoomName::from("room-a");
        app.insert_resource(state_with_lobby(lobby_with_room(
            &room,
            vec![occupant("alice")],
        )));
        app.update();
        let room_entity = room_entity(&mut app, &room);
        let (expected_prefix, expected_liveness) = DiscoveryState::room_prefix_and_lobby_status(
            Some(app.world().resource::<DiscoveryState>()),
            &game,
            &room,
        );
        assert_eq!(
            app.world().get::<p2p::HistoryKeyPrefix>(room_entity),
            Some(&expected_prefix)
        );
        assert_eq!(
            app.world()
                .get::<room_sync::state_machine::RoomLiveness>(room_entity),
            Some(&expected_liveness)
        );
    }
}
