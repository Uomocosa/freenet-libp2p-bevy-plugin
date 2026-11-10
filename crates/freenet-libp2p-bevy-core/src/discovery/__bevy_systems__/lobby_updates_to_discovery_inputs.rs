use bevy::prelude::*;

use crate::discovery;

pub fn lobby_updates_to_discovery_inputs(
    lobbies: Option<Res<discovery::LobbyUpdateReceiver>>,
    peers: Option<Res<discovery::FreenetPeersReceiver>>,
    mut inputs: MessageWriter<discovery::state_machine::Input>,
) {
    if let Some(lobbies) = lobbies.map(Res::into_inner) {
        let Ok(mut guard) = lobbies.lock() else {
            return;
        };
        let Some(rx) = guard.as_mut() else {
            return;
        };
        while let Ok(lobby) = rx.try_recv() {
            inputs.write(discovery::state_machine::Input::LobbyUpdated(lobby));
        }
    }
    if let Some(peers) = peers.map(Res::into_inner) {
        let Ok(mut guard) = peers.lock() else {
            return;
        };
        let Some(rx) = guard.as_mut() else {
            return;
        };
        while let Ok(list) = rx.try_recv() {
            inputs.write(discovery::state_machine::Input::FreenetPeers(list));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use bevy::prelude::*;

    use super::lobby_updates_to_discovery_inputs;
    use crate::discovery;
    use crate::identity;

    fn app_with_queues() -> (
        App,
        tokio::sync::mpsc::UnboundedSender<discovery::Lobby>,
        tokio::sync::mpsc::UnboundedSender<Vec<identity::FreenetPeer>>,
    ) {
        let (lobby_tx, lobby_rx) = tokio::sync::mpsc::unbounded_channel();
        let (peers_tx, peers_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::state_machine::Input>();
        app.insert_resource(discovery::LobbyUpdateReceiver(Mutex::new(Some(lobby_rx))));
        app.insert_resource(discovery::FreenetPeersReceiver(Mutex::new(Some(peers_rx))));
        app.add_systems(Update, lobby_updates_to_discovery_inputs);
        (app, lobby_tx, peers_tx)
    }

    #[test]
    fn test_usage() {
        let (mut app, lobby_tx, _) = app_with_queues();
        lobby_tx.send(discovery::Lobby::default()).expect("send");
        app.update();
        let inputs: Vec<discovery::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![discovery::state_machine::Input::LobbyUpdated(
                discovery::Lobby::default()
            )]
        );
    }

    #[test]
    fn freenet_peers_become_an_input() {
        let (mut app, _, peers_tx) = app_with_queues();
        let list = vec![identity::FreenetPeer {
            id: identity::FreenetPeerId::from("gateway"),
            addr: std::net::SocketAddr::from(([127, 0, 0, 1], 1)),
        }];
        peers_tx.send(list.clone()).expect("send");
        app.update();
        let inputs: Vec<discovery::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![discovery::state_machine::Input::FreenetPeers(list)]
        );
    }
}
