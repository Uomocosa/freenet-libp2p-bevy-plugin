use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;

use crate::discovery;
use crate::identity;

const NODE_STATUS_EVERY: u32 = 5;
const NODE_STATUS_HEARTBEAT: Duration = Duration::from_secs(30);

pub async fn poll_peers(
    state: &discovery::freenet::Config,
    peers_tx: UnboundedSender<Vec<identity::FreenetPeer>>,
) {
    let mut client =
        discovery::freenet::Client::from_host_and_port_with_retry(&state.host, state.port).await;
    let mut interval = tokio::time::interval(discovery::TICK);
    let mut ticks = 0_u32;
    let mut last_seen: Option<discovery::freenet::NodeStatus> = None;
    let mut last_logged = std::time::Instant::now();
    loop {
        interval.tick().await;
        let Ok(peers) = client.connected_peers().await else {
            continue;
        };
        if peers_tx.send(peers).is_err() {
            return;
        }
        ticks = ticks.wrapping_add(1);
        if ticks.rem_euclid(NODE_STATUS_EVERY) != 0 {
            continue;
        }
        let Ok(node) = client.node_status().await else {
            continue;
        };
        let has_changed = last_seen.as_ref() != Some(&node);
        if has_changed || last_logged.elapsed() >= NODE_STATUS_HEARTBEAT {
            tracing::info!(
                target: "discovery",
                ring_peers = node.ring_peers,
                transport_peers = node.transport_peers,
                ring_location = ?node.ring_location,
                "freenet node status"
            );
            last_seen = Some(node);
            last_logged = std::time::Instant::now();
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
