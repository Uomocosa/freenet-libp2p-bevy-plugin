use crate::discovery;
use crate::freenet;
use crate::p2p;

pub async fn run(
    config: &discovery::Config,
    endpoint: discovery::FreenetPort,
    mut swarm: p2p::SwarmChannels,
    target_rx: tokio::sync::watch::Receiver<Vec<discovery::state_machine::HeartbeatTarget>>,
    lobby_tx: tokio::sync::mpsc::UnboundedSender<discovery::Lobby>,
    peers_tx: tokio::sync::mpsc::UnboundedSender<Vec<freenet::FreenetPeer>>,
) {
    let Some(ready) = swarm.wait_ready().await else {
        return;
    };
    let freenet_config = discovery::LobbyTaskConfig {
        host: String::from("127.0.0.1"),
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
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::discovery;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let config = discovery::Config::default();
        let (libp2p_commands, _libp2p_rx) = tokio::sync::mpsc::unbounded_channel();
        let (libp2p_events, _event_rx) = tokio::sync::broadcast::channel(16);
        let (_ready_tx, ready_rx) = tokio::sync::watch::channel(None);
        let (_observed_tx, observed_rx) = tokio::sync::watch::channel(None);
        let channels = p2p::SwarmChannels {
            commands: libp2p_commands,
            events: libp2p_events.subscribe(),
            ready: ready_rx,
            observed: observed_rx,
        };
        let (_target_tx, target_rx) = tokio::sync::watch::channel(Vec::new());
        let (lobby_tx, _lobby_rx) = tokio::sync::mpsc::unbounded_channel();
        let (peers_tx, _peers_rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            run(
                &config,
                discovery::FreenetPort(0),
                channels,
                target_rx,
                lobby_tx,
                peers_tx,
            )
            .await;
        });
        handle.abort();
    }
}
