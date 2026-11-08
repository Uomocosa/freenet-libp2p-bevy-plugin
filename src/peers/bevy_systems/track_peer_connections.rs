use bevy::prelude::*;

use crate::p2p;
use crate::peers;

pub fn track_peer_connections(
    mut commands: Commands,
    mut tap: Option<ResMut<peers::PeerEventTap>>,
    mut peer_index: Option<ResMut<peers::PeerIndex>>,
) {
    let Some(tap) = tap.as_deref_mut() else {
        return;
    };
    let Some(peer_index) = peer_index.as_deref_mut() else {
        return;
    };
    while let Ok(event) = tap.try_recv() {
        match event {
            p2p::Libp2pEvent::PeerConnected(id)
            | p2p::Libp2pEvent::Libp2pConnectionEstablished { peer_id: id, .. } => {
                let entity = peer_index.get_or_spawn(&mut commands, &id);
                commands.entity(entity).insert(peers::Connected);
            }
            p2p::Libp2pEvent::PeerDisconnected(id) => {
                if let Some(entity) = peer_index.get(&id).copied() {
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
        app.insert_resource(peers::PeerIndex::default());
        app.add_systems(Update, track_peer_connections);
        (app, tx)
    }

    fn is_connected(app: &App, id: &identity::PeerId) -> bool {
        let entity = app.world().resource::<peers::PeerIndex>().get(id).copied();
        entity.is_some_and(|entity| app.world().get::<peers::Connected>(entity).is_some())
    }

    #[test]
    fn test_usage() {
        let (mut app, tx) = app_with_tap();
        let id = identity::PeerId::from("alice");
        tx.send(p2p::Libp2pEvent::PeerConnected(id.clone()))
            .expect("send");
        app.update();
        assert!(is_connected(&app, &id));
        tx.send(p2p::Libp2pEvent::PeerDisconnected(id.clone()))
            .expect("send");
        app.update();
        assert!(!is_connected(&app, &id));
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
        assert!(is_connected(&app, &id));
    }
}
