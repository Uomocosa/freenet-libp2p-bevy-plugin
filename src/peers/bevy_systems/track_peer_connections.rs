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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::track_peer_connections;
    use crate::identity;
    use crate::p2p;
    use crate::peers;

    fn app_with_tap() -> (App, tokio::sync::broadcast::Sender<p2p::Libp2pEvent>) {
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
        app.add_systems(Update, track_peer_connections);
        (app, tx)
    }

    fn is_connected(app: &mut App, id: &identity::PeerId) -> bool {
        let mut found = app.world_mut().query::<(Entity, &peers::Peer)>();
        let entity = found
            .iter(app.world())
            .find(|(_, peer)| *peer == id)
            .map(|(entity, _)| entity);
        entity.is_some_and(|entity| app.world().get::<peers::Connected>(entity).is_some())
    }

    fn peer_entity_count(app: &mut App, id: &identity::PeerId) -> usize {
        app.world_mut()
            .query::<(Entity, &peers::Peer)>()
            .iter(app.world())
            .filter(|(_, peer)| peer == &id)
            .count()
    }

    #[test]
    fn test_usage() {
        let (mut app, tx) = app_with_tap();
        let id = identity::PeerId::from("alice");
        tx.send(p2p::Libp2pEvent::PeerConnected(id.clone()))
            .expect("send");
        app.update();
        assert!(is_connected(&mut app, &id));
        tx.send(p2p::Libp2pEvent::PeerDisconnected(id.clone()))
            .expect("send");
        app.update();
        assert!(!is_connected(&mut app, &id));
    }

    #[test]
    fn connection_established_counts_as_connected() {
        let (mut app, tx) = app_with_tap();
        let id = identity::PeerId::from("bob");
        tx.send(p2p::Libp2pEvent::Libp2pConnectionEstablished {
            peer_id: id.clone(),
            kind: p2p::ConnectionKind::Tcp,
        })
        .expect("send");
        app.update();
        assert!(is_connected(&mut app, &id));
    }

    #[test]
    fn duplicate_connect_is_idempotent() {
        let (mut app, tx) = app_with_tap();
        let id = identity::PeerId::from("alice");
        tx.send(p2p::Libp2pEvent::PeerConnected(id.clone()))
            .expect("send");
        app.update();
        tx.send(p2p::Libp2pEvent::PeerConnected(id.clone()))
            .expect("send");
        app.update();
        assert!(is_connected(&mut app, &id));
        assert_eq!(peer_entity_count(&mut app, &id), 1);
    }

    #[test]
    fn disconnect_of_unknown_peer_is_a_no_op() {
        let (mut app, tx) = app_with_tap();
        tx.send(p2p::Libp2pEvent::PeerDisconnected(identity::PeerId::from(
            "ghost",
        )))
        .expect("send");
        app.update();
        assert_eq!(
            peer_entity_count(&mut app, &identity::PeerId::from("ghost")),
            0
        );
    }
}
