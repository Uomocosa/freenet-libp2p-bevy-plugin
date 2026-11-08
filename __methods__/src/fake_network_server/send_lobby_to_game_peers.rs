use std::sync::Arc;

use crate::discovery;
use crate::identity;
use crate::testing;

pub async fn send_lobby_to_game_peers(
    task: &testing::FakeNetworkServer,
    shared: &Arc<tokio::sync::Mutex<testing::orchestrator_shared::OrchestratorShared>>,
    peer: &identity::PeerId,
    lobby: discovery::Lobby,
) {
    let _ = *task;
    let targets = {
        let guard = shared.lock().await;
        let Some(game) = guard.peer_games.get(peer).cloned() else {
            return;
        };
        guard
            .peer_games
            .iter()
            .filter(|(_, peer_game)| *peer_game == &game)
            .map(|(peer, _)| peer.clone())
            .collect::<Vec<_>>()
    };
    for peer in targets {
        let senders = {
            let guard = shared.lock().await;
            guard.senders.get(&peer).cloned().unwrap_or_default()
        };
        for sender in senders {
            sender
                .send(testing::FakeWireServerFrame::Lobby(lobby.clone()))
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_usage() {
        let _ = stringify!(send_lobby_to_game_peers);
    }
}
