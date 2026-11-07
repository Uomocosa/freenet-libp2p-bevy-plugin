use crate::discovery;

pub async fn spawn_lobby_tasks(
    state: &discovery::freenet::Config,
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

#[cfg(test)]
mod tests {
    use super::spawn_lobby_tasks;
    use crate::discovery;
    use crate::identity;

    #[tokio::test]
    async fn test_usage() {
        let config = discovery::freenet::Config {
            host: "127.0.0.1".to_string(),
            port: 0,
            me: identity::PeerId::from("me"),
            params: Vec::new(),
            discovery: discovery::Config::default(),
        };
        let (_target_tx, target_rx) = tokio::sync::watch::channel(Vec::new());
        let (lobbies_tx, _lobbies_rx) = tokio::sync::mpsc::unbounded_channel();
        let (peers_tx, _peers_rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            spawn_lobby_tasks(
                &config,
                discovery::LobbyTaskChannels {
                    heartbeat_targets: target_rx,
                    lobbies: lobbies_tx,
                    freenet_peers: peers_tx,
                },
            )
            .await;
        });
        handle.abort();
    }
}
