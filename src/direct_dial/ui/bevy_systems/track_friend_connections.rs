use bevy::prelude::*;

use crate::direct_dial;

pub fn track_friend_connections(
    mut tap: Option<ResMut<direct_dial::FriendEventTap>>,
    mut connected: ResMut<direct_dial::ConnectedPeers>,
) {
    let Some(tap) = tap.as_deref_mut() else {
        return;
    };
    while let Ok(event) = tap.try_recv() {
        connected.apply_connection_event(&event);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::track_friend_connections;
    use crate::direct_dial;
    use crate::identity;
    use crate::p2p;

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
        app.insert_resource(direct_dial::FriendEventTap(rx));
        app.insert_resource(direct_dial::ConnectedPeers::default());
        app.add_systems(Update, track_friend_connections);
        (app, tx)
    }

    #[test]
    fn test_usage() {
        let (mut app, tx) = app_with_tap();
        let id = identity::PeerId::from("alice");
        tx.send(p2p::Libp2pEvent::PeerConnected(id.clone()))
            .expect("send");
        app.update();
        assert!(
            app.world()
                .resource::<direct_dial::ConnectedPeers>()
                .contains(&id)
        );
        tx.send(p2p::Libp2pEvent::PeerDisconnected(id.clone()))
            .expect("send");
        app.update();
        assert!(
            !app.world()
                .resource::<direct_dial::ConnectedPeers>()
                .contains(&id)
        );
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
        assert!(
            app.world()
                .resource::<direct_dial::ConnectedPeers>()
                .contains(&id)
        );
    }
}
