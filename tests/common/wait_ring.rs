use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::{log_contains, poke};

const RING_READY: &str = "lobby freenet ring ready";
const POLL_MS: u64 = 500;

/// # Returns
/// One flag per log, in order: did that peer's embedded Freenet node join the ring?
#[must_use]
pub fn did_rings_become_ready(logs: &[PathBuf], timeout: Duration) -> Vec<bool> {
    let start = Instant::now();
    loop {
        let ready: Vec<bool> = logs
            .iter()
            .map(|log| log_contains(log, RING_READY))
            .collect();
        if ready.iter().all(|ok| *ok) || start.elapsed() >= timeout {
            return ready;
        }
        poke();
        std::thread::sleep(Duration::from_millis(POLL_MS));
    }
}
