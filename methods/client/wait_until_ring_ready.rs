use std::time::Duration;

use crate::discovery;

pub async fn wait_until_ring_ready(
    state: &mut discovery::freenet::Client,
    budget: Duration,
) -> Result<(), discovery::Error> {
    let tick = Duration::from_secs(1);
    let started = std::time::Instant::now();
    loop {
        let node = state.node_status().await?;
        if node.is_ring_ready() {
            tracing::info!(
                target: "discovery",
                ring_peers = node.ring_peers,
                transport_peers = node.transport_peers,
                ring_location = ?node.ring_location,
                waited_ms = started.elapsed().as_millis(),
                "lobby freenet ring ready"
            );
            return Ok(());
        }
        let waited = started.elapsed();
        if waited >= budget {
            tracing::warn!(
                target: "discovery",
                ring_peers = node.ring_peers,
                transport_peers = node.transport_peers,
                ring_location = ?node.ring_location,
                waited_ms = waited.as_millis(),
                budget_ms = budget.as_millis(),
                "lobby freenet ring not ready, proceeding via gateway"
            );
            return Ok(());
        }
        tracing::info!(
            target: "discovery",
            ring_peers = node.ring_peers,
            transport_peers = node.transport_peers,
            ring_location = ?node.ring_location,
            waited_ms = waited.as_millis(),
            "lobby freenet ring waiting"
        );
        tokio::time::sleep(tick).await;
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
