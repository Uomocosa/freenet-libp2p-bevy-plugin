use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;

use crate::direct_dial;
use crate::identity;
use crate::peers;

pub fn project_friends_to_entities(
    mut commands: Commands,
    book: Option<Res<direct_dial::PeerBook>>,
    found_peers: Query<(Entity, &peers::Peer)>,
    friends: Query<(Entity, &peers::Peer, Has<peers::KeepAlive>), With<direct_dial::Friend>>,
) {
    let Some(book) = book.map(Res::into_inner) else {
        return;
    };
    let friend_ids: BTreeSet<identity::PeerId> = book
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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::project_friends_to_entities;
    use crate::direct_dial;
    use crate::identity;
    use crate::p2p;
    use crate::peers;

    fn app_with_projection(book: direct_dial::PeerBook) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(book);
        app.add_systems(Update, project_friends_to_entities);
        app
    }

    fn friend_book(names: &[&str]) -> direct_dial::PeerBook {
        let mut book = direct_dial::PeerBook::default();
        for name in names {
            book.add_friend(identity::Peer {
                id: identity::PeerId::from(*name),
                addrs: Vec::new(),
            });
        }
        book
    }

    fn friend_entity(app: &mut App, id: &str) -> Entity {
        let mut found = app.world_mut().query::<(Entity, &peers::Peer)>();
        found
            .iter(app.world())
            .find(|(_, peer)| **peer == identity::PeerId::from(id))
            .map(|(entity, _)| entity)
            .expect("peer exists")
    }

    fn peer_exists(app: &mut App, id: &str) -> bool {
        let mut found = app.world_mut().query::<(Entity, &peers::Peer)>();
        found
            .iter(app.world())
            .any(|(_, peer)| **peer == identity::PeerId::from(id))
    }

    #[test]
    fn test_usage() {
        let mut app = app_with_projection(friend_book(&["alice"]));
        app.update();
        let alice = friend_entity(&mut app, "alice");
        assert!(app.world().get::<direct_dial::Friend>(alice).is_some());
        assert!(app.world().get::<peers::KeepAlive>(alice).is_some());
    }

    #[test]
    fn friend_removed_loses_friend_and_keep_alive() {
        let mut app = app_with_projection(friend_book(&["alice"]));
        app.update();
        let alice = friend_entity(&mut app, "alice");
        assert!(app.world().get::<direct_dial::Friend>(alice).is_some());
        app.world_mut()
            .resource_mut::<direct_dial::PeerBook>()
            .remove_friend(&identity::PeerId::from("alice"));
        app.update();
        assert!(app.world().get::<direct_dial::Friend>(alice).is_none());
        assert!(app.world().get::<peers::KeepAlive>(alice).is_none());
    }

    #[test]
    fn previously_connected_peer_is_not_a_friend() {
        let mut book = direct_dial::PeerBook::default();
        book.push(direct_dial::KnownPeer {
            peer: identity::Peer {
                id: identity::PeerId::from("alice"),
                addrs: Vec::new(),
            },
            kind: direct_dial::KnownPeerKind::PreviouslyConnected,
        });
        let mut app = app_with_projection(book);
        app.update();
        assert!(!peer_exists(&mut app, "alice"));
    }

    #[test]
    fn friend_peer_is_not_despawned_while_disconnected() {
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
        app.insert_resource(friend_book(&["alice"]));
        app.add_systems(
            Update,
            (
                peers::bevy_systems::track_peer_connections,
                project_friends_to_entities,
                peers::bevy_systems::despawn_orphan_peers,
            )
                .chain(),
        );
        let id = identity::PeerId::from("alice");
        tx.send(p2p::Libp2pEvent::PeerConnected(id.clone()))
            .expect("send");
        app.update();
        let alice = friend_entity(&mut app, "alice");
        assert!(app.world().get::<peers::Connected>(alice).is_some());
        assert!(app.world().get::<direct_dial::Friend>(alice).is_some());
        tx.send(p2p::Libp2pEvent::PeerDisconnected(id))
            .expect("send");
        app.update();
        app.update();
        assert!(app.world().get_entity(alice).is_ok());
        assert!(app.world().get::<direct_dial::Friend>(alice).is_some());
        assert!(app.world().get::<peers::Connected>(alice).is_none());
    }
}
