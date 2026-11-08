use bevy::prelude::*;

use crate::peers;

type OrphanPeerRow = (
    Entity,
    Has<peers::Connected>,
    Has<peers::InRoom>,
    Has<peers::HeldRooms>,
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
            |(_, is_connected, is_in_room, holds_history, is_keep_alive, is_me)| {
                !is_connected && !is_in_room && !holds_history && !is_keep_alive && !is_me
            },
        )
        .map(|(entity, _, _, _, _, _)| entity)
        .collect();
    for entity in orphans {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::despawn_orphan_peers;
    use crate::identity;
    use crate::p2p;
    use crate::peers;

    fn app_with_system() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, despawn_orphan_peers);
        app
    }

    fn peer_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<peers::Peer>>()
            .iter(app.world())
            .count()
    }

    #[test]
    fn test_usage() {
        let mut app = app_with_system();
        let orphan = app
            .world_mut()
            .spawn(peers::Peer::from(identity::PeerId::from("orphan")))
            .id();
        let kept = app
            .world_mut()
            .spawn((
                peers::Peer::from(identity::PeerId::from("kept")),
                peers::Connected,
            ))
            .id();
        app.update();
        assert_eq!(peer_count(&mut app), 1);
        assert!(app.world().get_entity(orphan).is_err());
        assert!(app.world().get_entity(kept).is_ok());
    }

    #[test]
    fn orphan_peer_despawned_after_disconnect() {
        let channel = p2p::Libp2pEventChannel::default();
        let tx = channel.tx.clone();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(channel);
        let rx = app
            .world()
            .resource::<p2p::Libp2pEventChannel>()
            .subscribe();
        app.insert_resource(peers::PeerEventTap(rx));
        app.add_systems(
            Update,
            (
                peers::bevy_systems::track_peer_connections,
                despawn_orphan_peers,
            )
                .chain(),
        );
        let id = identity::PeerId::from("alice");
        tx.send(p2p::Libp2pEvent::PeerConnected(id.clone()))
            .expect("send");
        app.update();
        assert_eq!(peer_count(&mut app), 1);
        tx.send(p2p::Libp2pEvent::PeerDisconnected(id))
            .expect("send");
        app.update();
        app.update();
        assert_eq!(peer_count(&mut app), 0);
    }

    #[test]
    fn peer_in_room_is_kept() {
        let mut app = app_with_system();
        let room = app
            .world_mut()
            .spawn(peers::Room::from(identity::RoomName::from("lobby")))
            .id();
        let member = app
            .world_mut()
            .spawn((
                peers::Peer::from(identity::PeerId::from("member")),
                peers::InRoom(room),
            ))
            .id();
        app.update();
        assert!(app.world().get_entity(member).is_ok());
        assert!(
            app.world()
                .get::<peers::RoomMembers>(room)
                .is_some_and(|members| members.contains(&member))
        );
    }
}
