use bevy::prelude::*;

use crate::identity;
use crate::peers;

type OrphanPeerRow<'a> = (
    Entity,
    &'a peers::Peer,
    Has<peers::Connected>,
    Has<peers::InRoom>,
    Has<peers::HoldsHistoryOf>,
    Has<peers::Me>,
);

pub fn despawn_orphan_peers(
    mut commands: Commands,
    found_peers: Query<OrphanPeerRow<'_>>,
    mut peer_index: Option<ResMut<peers::PeerIndex>>,
) {
    let Some(peer_index) = peer_index.as_deref_mut() else {
        return;
    };
    let orphans: Vec<(Entity, identity::PeerId)> = found_peers
        .iter()
        .filter(|(_, _, is_connected, is_in_room, holds_history, is_me)| {
            !is_connected && !is_in_room && !holds_history && !is_me
        })
        .map(|(entity, peer, _, _, _, _)| (entity, (**peer).clone()))
        .collect();
    for (entity, id) in orphans {
        peer_index.remove(&id);
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
        app.init_resource::<peers::PeerIndex>();
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
        app.world_mut()
            .resource_mut::<peers::PeerIndex>()
            .insert(identity::PeerId::from("orphan"), orphan);
        let kept = app
            .world_mut()
            .spawn((
                peers::Peer::from(identity::PeerId::from("kept")),
                peers::Connected,
            ))
            .id();
        app.world_mut()
            .resource_mut::<peers::PeerIndex>()
            .insert(identity::PeerId::from("kept"), kept);
        app.update();
        assert_eq!(peer_count(&mut app), 1);
        assert!(app.world().get_entity(orphan).is_err());
        assert!(app.world().get_entity(kept).is_ok());
        assert!(
            app.world()
                .resource::<peers::PeerIndex>()
                .get(&identity::PeerId::from("orphan"))
                .is_none()
        );
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
        app.insert_resource(peers::PeerIndex::default());
        app.add_systems(
            Update,
            (
                peers::bevy_systems::track_peer_connections,
                despawn_orphan_peers,
            ),
        );
        let id = identity::PeerId::from("alice");
        tx.send(p2p::Libp2pEvent::PeerConnected(id.clone()))
            .expect("send");
        app.update();
        assert_eq!(peer_count(&mut app), 1);
        tx.send(p2p::Libp2pEvent::PeerDisconnected(id.clone()))
            .expect("send");
        app.update();
        app.update();
        assert_eq!(peer_count(&mut app), 0);
        assert!(
            app.world()
                .resource::<peers::PeerIndex>()
                .get(&id)
                .is_none()
        );
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
