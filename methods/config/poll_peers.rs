use tokio::sync::mpsc::UnboundedSender;

use crate::discovery;
use crate::identity;

pub async fn poll_peers(
    state: &discovery::freenet::Config,
    peers_tx: UnboundedSender<Vec<identity::FreenetPeer>>,
) {
    let mut client = discovery::freenet::Client::connect_socket(&state.host, state.port).await;
    let mut interval = tokio::time::interval(discovery::TICK);
    loop {
        interval.tick().await;
        if let Ok(peers) = client.connected_peers().await
            && peers_tx.send(peers).is_err()
        {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::poll_peers;

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
        let (peers_tx, _peers_rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::spawn(async move { poll_peers(&config, peers_tx).await });
        handle.abort();
    }
}
