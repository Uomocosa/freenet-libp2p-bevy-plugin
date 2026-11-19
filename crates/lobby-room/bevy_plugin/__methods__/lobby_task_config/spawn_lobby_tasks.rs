use crate::discovery;

pub async fn spawn_lobby_tasks(
    state: &discovery::LobbyTaskConfig,
    channels: discovery::LobbyTaskChannels,
) {
    let discovery::LobbyTaskChannels {
        heartbeat_targets,
        lobbies,
        freenet_peers,
    } = channels;
    let peers_config = state.clone();
    tokio::spawn(async move { peers_config.poll_peers(freenet_peers).await });
    let freenet_config = state.clone();
    tokio::spawn(async move {
        let client = freenet_config.connect_retry(&heartbeat_targets).await;
        freenet_config
            .run_lobby_sync_loop(client, heartbeat_targets, lobbies)
            .await;
    });
}

// no test_usage necessary
