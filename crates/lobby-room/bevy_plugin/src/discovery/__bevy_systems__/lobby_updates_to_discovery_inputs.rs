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
