use std::time::Duration;

use crate::discovery;
use discovery::freenet::basic::constants::MIN_RING_PEERS;

pub async fn wait_until_ring_ready(
    client: &mut discovery::freenet::Client,
    budget: Duration,
) -> Result<(), discovery::Error> {
    let tick = Duration::from_secs(1);
    let mut waited = Duration::ZERO;
    loop {
        let peers = discovery::freenet::connected_peers(client).await?;
        let addrs: Vec<String> = peers.iter().map(|p| p.addr.to_string()).collect();
        if discovery::freenet::is_ring_ready(&peers, MIN_RING_PEERS) {
            tracing::info!(target: "discovery", ?addrs, "lobby freenet ring-ready gate peer addrs");
            tracing::info!(
                target: "discovery",
                "lobby freenet transport connected peers={} waited_ms={}",
                peers.len(),
                waited.as_millis()
            );
            return Ok(());
        }
        if waited >= budget {
            tracing::warn!(
                target: "discovery",
                ?addrs,
                "lobby freenet ring not ready, proceeding via gateway peers={} waited_ms={} budget_ms={}",
                peers.len(),
                waited.as_millis(),
                budget.as_millis()
            );
            return Ok(());
        }
        tokio::time::sleep(tick).await;
        waited = waited.saturating_add(tick);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::wait_until_ring_ready;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = discovery::freenet::Client { write, read };
        let result = wait_until_ring_ready(&mut client, Duration::from_millis(1)).await;
        assert!(result.is_err());
    }
}
