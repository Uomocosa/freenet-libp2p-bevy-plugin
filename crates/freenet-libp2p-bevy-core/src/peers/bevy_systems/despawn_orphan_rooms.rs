use std::collections::HashSet;

use bevy::prelude::*;

use crate::identity;
use crate::peers;

pub fn despawn_orphan_rooms(
    mut commands: Commands,
    found_rooms: Query<(
        Entity,
        &identity::RoomName,
        Option<&peers::RoomMembers>,
        Option<&peers::HistoryHolders>,
    )>,
    me_rooms: Query<&peers::InRoom, With<peers::Me>>,
) {
    let held_by_me: HashSet<Entity> = me_rooms.iter().map(|in_room| **in_room).collect();
    let orphans: Vec<Entity> = found_rooms
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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::despawn_orphan_rooms;
    use crate::identity;
    use crate::peers;

    fn app_with_system() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, despawn_orphan_rooms);
        app
    }

    fn room_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<identity::RoomName>>()
            .iter(app.world())
            .count()
    }

    #[test]
    fn test_usage() {
        let mut app = app_with_system();
        let orphan = app
            .world_mut()
            .spawn(identity::RoomName::from("empty"))
            .id();
        let lived = app
            .world_mut()
            .spawn(identity::RoomName::from("lived"))
            .id();
        app.world_mut().spawn((
            peers::Peer::from(identity::PeerId::from("member")),
            peers::InRoom(lived),
        ));
        app.update();
        assert_eq!(room_count(&mut app), 1);
        assert!(app.world().get_entity(orphan).is_err());
        assert!(app.world().get_entity(lived).is_ok());
    }

    #[test]
    fn in_room_fills_room_members() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let room = app
            .world_mut()
            .spawn(identity::RoomName::from("lobby"))
            .id();
        let member = app
            .world_mut()
            .spawn((
                peers::Peer::from(identity::PeerId::from("alice")),
                peers::InRoom(room),
            ))
            .id();
        let members = app
            .world()
            .get::<peers::RoomMembers>(room)
            .expect("members");
        assert_eq!(members.len(), 1);
        assert!(members.contains(&member));
    }

    #[test]
    fn me_room_is_kept() {
        let mut app = app_with_system();
        let room = app.world_mut().spawn(identity::RoomName::from("home")).id();
        app.world_mut().spawn((
            peers::Peer::from(identity::PeerId::from("me")),
            peers::Me,
            peers::InRoom(room),
        ));
        app.update();
        assert!(app.world().get_entity(room).is_ok());
    }

    #[test]
    fn room_kept_only_by_a_holder_survives_the_sweep() {
        let mut app = app_with_system();
        let room = app
            .world_mut()
            .spawn(identity::RoomName::from("archive"))
            .id();
        let holder = app
            .world_mut()
            .spawn(peers::Peer::from(identity::PeerId::from("holder")))
            .id();
        app.world_mut().spawn((
            peers::HistoryHold,
            peers::HeldBy(holder),
            peers::HoldOf(room),
        ));
        app.update();
        app.update();
        assert!(app.world().get_entity(room).is_ok());
    }

    #[test]
    fn despawned_peer_takes_its_links_with_it() {
        let mut app = app_with_system();
        let room = app
            .world_mut()
            .spawn(identity::RoomName::from("archive"))
            .id();
        let holder = app
            .world_mut()
            .spawn(peers::Peer::from(identity::PeerId::from("holder")))
            .id();
        let link = app
            .world_mut()
            .spawn((
                peers::HistoryHold,
                peers::HeldBy(holder),
                peers::HoldOf(room),
            ))
            .id();
        app.update();
        assert!(app.world().get_entity(link).is_ok());
        app.world_mut().despawn(holder);
        app.update();
        assert!(app.world().get_entity(link).is_err());
        app.update();
        app.update();
        assert!(app.world().get_entity(room).is_err());
    }

    #[test]
    fn room_follows_its_last_holder() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(
            Update,
            (
                peers::bevy_systems::despawn_orphan_peers,
                despawn_orphan_rooms,
            )
                .chain(),
        );
        let room = app
            .world_mut()
            .spawn(identity::RoomName::from("archive"))
            .id();
        let holder = app
            .world_mut()
            .spawn(peers::Peer::from(identity::PeerId::from("holder")))
            .id();
        let link = app
            .world_mut()
            .spawn((
                peers::HistoryHold,
                peers::HeldBy(holder),
                peers::HoldOf(room),
            ))
            .id();
        app.update();
        assert!(app.world().get_entity(room).is_ok());
        assert!(
            app.world()
                .get::<peers::HistoryHolders>(room)
                .is_some_and(|holders| holders.contains(&link))
        );
        app.world_mut().despawn(holder);
        app.update();
        app.update();
        app.update();
        assert!(app.world().get_entity(room).is_err());
    }
}
