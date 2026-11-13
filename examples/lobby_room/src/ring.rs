use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::lobby_room::freenet;

/// How many ring peers the embedded Freenet node currently reports.
///
/// The plugin's `Status` only carries transport peers (`freenet_peers`), so the
/// example polls `node_status` itself every 500 ms and shares the count here.
/// `run_script` reads it for `wait-ring-joined`.
#[derive(Resource, Debug, Clone)]
pub struct RingPeers {
    count: Arc<AtomicUsize>,
}

impl RingPeers {
    #[must_use]
    pub fn new() -> Self {
        Self {
            count: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[must_use]
    pub fn get(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    pub fn set(&self, count: usize) {
        self.count.store(count, Ordering::Relaxed);
    }
}

impl Default for RingPeers {
    fn default() -> Self {
        Self::new()
    }
}

/// Polls the embedded node's `ring_peers` forever, sharing the count via `ring`.
/// Logs `ring joined ring_peers=N` once at info before storing, so
/// `did_every_peer_join_the_ring` (`max_ring_peers`) sees it even when the
/// script exits soon after joining.
pub fn spawn_ring_poller(ws_port: u16, ring: RingPeers) {
    tokio::spawn(async move {
        let mut client = freenet::Client::from_host_and_port_with_retry("127.0.0.1", ws_port).await;
        let mut has_logged_join = false;
        loop {
            if let Ok(status) = client.node_status().await {
                let count = status.ring_peers;
                if count > 0 && !has_logged_join {
                    tracing::info!("ring joined ring_peers={count}");
                    has_logged_join = true;
                }
                ring.set(count);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::RingPeers;

    #[test]
    fn test_usage() {
        let ring = RingPeers::new();
        assert_eq!(ring.get(), 0);
        ring.set(3);
        assert_eq!(ring.get(), 3);
        assert_eq!(RingPeers::default().get(), 0);
    }
}
