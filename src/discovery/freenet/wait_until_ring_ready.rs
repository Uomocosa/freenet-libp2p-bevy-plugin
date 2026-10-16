use std::time::Duration;

use crate::discovery;

pub async fn wait_until_ring_ready(
    client: &mut discovery::freenet::Client,
    budget: Duration,
) -> Result<(), discovery::Error> {
    let tick = Duration::from_secs(1);
    let mut waited = Duration::ZERO;
    loop {
        let peers = discovery::freenet::connected_peer_count(client).await?;
        if peers > 0 {
            tracing::info!(
                target: "room_lobby",
                "lobby freenet transport connected peers={peers} waited_ms={}",
                waited.as_millis()
            );
            return Ok(());
        }
        if waited >= budget {
            tracing::warn!(
                target: "room_lobby",
                "lobby freenet ring not ready waited_ms={} budget_ms={}",
                waited.as_millis(),
                budget.as_millis()
            );
            return Err(discovery::Error::ConnectionTimeout);
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
