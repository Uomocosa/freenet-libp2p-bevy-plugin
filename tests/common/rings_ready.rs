use std::path::PathBuf;

use super::log_contains;

const RING_READY: &str = "lobby freenet ring ready";

/// # Returns
/// One flag per log, in order: has that peer's embedded Freenet node read the
/// lobby yet? The `ring ready` marker is emitted by `attach` only once the node
/// has fetched the lobby, so it distinguishes "node is isolated" from "lobby row
/// has not propagated" instead of firing on the transient gateway handshake.
#[must_use]
pub fn rings_ready(logs: &[PathBuf]) -> Vec<bool> {
    logs.iter()
        .map(|log| log_contains(log, RING_READY))
        .collect()
}
