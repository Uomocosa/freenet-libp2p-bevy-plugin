use std::path::PathBuf;

use super::log_contains;

const NODE_REACHABLE: &str = "lobby freenet node reachable";

/// # Returns
/// One flag per log, in order: has that peer's embedded Freenet node reached the network yet?
/// The `node reachable` marker is emitted by `wait_until_node_reachable` once the node reports a
/// transport or ring connection, so it distinguishes "node is isolated" from "lobby row has not
/// propagated".
#[must_use]
pub fn nodes_reachable(logs: &[PathBuf]) -> Vec<bool> {
    logs.iter()
        .map(|log| log_contains(log, NODE_REACHABLE))
        .collect()
}
