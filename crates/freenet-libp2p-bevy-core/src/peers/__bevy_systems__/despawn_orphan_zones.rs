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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::despawn_orphan_zones;
    use crate::identity;
    use crate::peers;

    fn app_with_system() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, despawn_orphan_zones);
        app
    }

    fn zone_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<identity::ZoneName>>()
            .iter(app.world())
            .count()
    }

    #[test]
    fn test_usage() {
        let mut app = app_with_system();
        let orphan = app
            .world_mut()
            .spawn(identity::ZoneName::from("empty"))
            .id();
        let lived = app
            .world_mut()
            .spawn(identity::ZoneName::from("lived"))
            .id();
        app.world_mut().spawn((
            peers::Peer::from(identity::PeerId::from("member")),
            peers::InZone(lived),
        ));
        app.update();
        assert_eq!(zone_count(&mut app), 1);
        assert!(app.world().get_entity(orphan).is_err());
        assert!(app.world().get_entity(lived).is_ok());
    }

    #[test]
    fn in_zone_fills_zone_members() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let zone = app
            .world_mut()
            .spawn(identity::ZoneName::from("global"))
            .id();
        let member = app
            .world_mut()
            .spawn((
                peers::Peer::from(identity::PeerId::from("alice")),
                peers::InZone(zone),
            ))
            .id();
        let members = app
            .world()
            .get::<peers::ZoneMembers>(zone)
            .expect("members");
        assert_eq!(members.len(), 1);
        assert!(members.contains(&member));
    }

    #[test]
    fn me_zone_is_kept() {
        let mut app = app_with_system();
        let zone = app.world_mut().spawn(identity::ZoneName::from("home")).id();
        app.world_mut().spawn((
            peers::Peer::from(identity::PeerId::from("me")),
            peers::Me,
            peers::InZone(zone),
        ));
        app.update();
        assert!(app.world().get_entity(zone).is_ok());
    }

    #[test]
    fn zone_kept_only_by_a_holder_survives_the_sweep() {
        let mut app = app_with_system();
        let zone = app
            .world_mut()
            .spawn(identity::ZoneName::from("archive"))
            .id();
        let holder = app
            .world_mut()
            .spawn(peers::Peer::from(identity::PeerId::from("holder")))
            .id();
        app.world_mut().spawn((
            peers::HistoryHold,
            peers::HeldBy(holder),
            peers::HoldOf(zone),
        ));
        app.update();
        app.update();
        assert!(app.world().get_entity(zone).is_ok());
    }

    #[test]
    fn despawned_peer_takes_its_links_with_it() {
        let mut app = app_with_system();
        let zone = app
            .world_mut()
            .spawn(identity::ZoneName::from("archive"))
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
                peers::HoldOf(zone),
            ))
            .id();
        app.update();
        assert!(app.world().get_entity(link).is_ok());
        app.world_mut().despawn(holder);
        app.update();
        assert!(app.world().get_entity(link).is_err());
        app.update();
        app.update();
        assert!(app.world().get_entity(zone).is_err());
    }

    #[test]
    fn zone_follows_its_last_holder() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(
            Update,
            (
                peers::bevy_systems::despawn_orphan_peers,
                despawn_orphan_zones,
            )
                .chain(),
        );
        let zone = app
            .world_mut()
            .spawn(identity::ZoneName::from("archive"))
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
                peers::HoldOf(zone),
            ))
            .id();
        app.update();
        assert!(app.world().get_entity(zone).is_ok());
        assert!(
            app.world()
                .get::<peers::HistoryHolders>(zone)
                .is_some_and(|holders| holders.contains(&link))
        );
        app.world_mut().despawn(holder);
        app.update();
        app.update();
        app.update();
        assert!(app.world().get_entity(zone).is_err());
    }
}
