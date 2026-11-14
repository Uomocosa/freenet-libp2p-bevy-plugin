use std::time::Duration;

use e2e_apps::starboard::CanvasStatus;

use super::{StarboardPeer, poll_until};

/// Polls until every peer has painted tiles, a canvas stable for `quiet`, and one shared hash.
///
/// # Errors
/// Returns an error listing the last observed statuses once `deadline` passes.
pub async fn expect_canvases_converge(
    peers: &[&StarboardPeer],
    quiet: Duration,
    deadline: Duration,
) -> Result<Vec<CanvasStatus>, String> {
    poll_until(
        "canvases converge",
        deadline,
        async || {
            let mut statuses = Vec::with_capacity(peers.len());
            for peer in peers {
                statuses.push(peer.canvas_status().await?);
            }
            Ok(statuses)
        },
        |statuses: &Vec<CanvasStatus>| {
            let quiet_ms = quiet.as_millis();
            let first_hash = statuses.first().map(|status| status.hash);
            !statuses.is_empty()
                && statuses.iter().all(|status| {
                    status.painted_tiles > 0
                        && u128::from(status.stable_for_ms) >= quiet_ms
                        && Some(status.hash) == first_hash
                })
        },
    )
    .await
}
