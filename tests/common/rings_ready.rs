use std::path::PathBuf;

use super::log_contains;

const RING_READY: &str = "lobby freenet ring ready";

/// # Returns
/// One flag per log, in order: has that peer's embedded Freenet node reached the ring yet?
/// The `ring ready` marker is emitted by `wait_until_ring_ready` only once the node reports a
/// non-gateway ring connection, so it distinguishes "node is isolated" from "lobby row has not
/// propagated" instead of matching the transient gateway transport link.
#[must_use]
pub fn rings_ready(logs: &[PathBuf]) -> Vec<bool> {
    logs.iter()
        .map(|log| log_contains(log, RING_READY))
        .collect()
}
