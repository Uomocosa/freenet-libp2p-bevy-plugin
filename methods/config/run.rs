use crate::discovery;
use crate::identity;

pub async fn run(
    config: &discovery::Config,
    endpoint: discovery::FreenetPort,
    mut channels: discovery::RunChannels,
) {
    let (target_tx, target_rx) = tokio::sync::watch::channel(Vec::new());
    let (lobby_tx, lobbies) = tokio::sync::mpsc::unbounded_channel();
    let (peers_tx, peers) = tokio::sync::mpsc::unbounded_channel();
    let Some(ready) = identity::Peer::wait_ready(&mut channels.swarm.ready).await else {
        return;
    };
    let freenet_config = discovery::freenet::Config {
        host: "127.0.0.1".to_string(),
        port: *endpoint,
        me: ready.id.clone(),
        params: config.game.contract_params(),
        discovery: config.clone(),
    };
    freenet_config
        .spawn_lobby_tasks(discovery::LobbyTaskChannels {
            heartbeat_targets: target_rx,
            lobbies: lobby_tx,
            freenet_peers: peers_tx,
        })
        .await;
    config
        .run_event_loop(channels, target_tx, lobbies, peers)
        .await;
}

#[cfg(test)]
mod tests {
    use super::run;

    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let config = discovery::Config::default();
        let (libp2p_commands, _libp2p_rx) = tokio::sync::mpsc::unbounded_channel();
        let (libp2p_events, _libp2p_event_rx) = tokio::sync::broadcast::channel(16);
        let (_ready_tx, ready_rx) = tokio::sync::watch::channel(None);
        let (_observed_tx, observed_rx) = tokio::sync::watch::channel(None);
        let (_commands_tx, commands_rx) = tokio::sync::mpsc::unbounded_channel();
        let (statuses_tx, _statuses_rx) = tokio::sync::mpsc::unbounded_channel();
        let (events_tx, _events_rx) = tokio::sync::mpsc::unbounded_channel();
        let channels = discovery::RunChannels {
            swarm: discovery::libp2p::SwarmChannels {
                commands: libp2p_commands,
                events: libp2p_events.subscribe(),
                ready: ready_rx,
                observed: observed_rx,
            },
            commands: commands_rx,
            statuses: statuses_tx,
            events: events_tx,
        };
        let handle = tokio::spawn(async move {
            run(&config, discovery::FreenetPort(0), channels).await;
        });
        handle.abort();
    }
}
