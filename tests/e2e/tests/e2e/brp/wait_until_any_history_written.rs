use std::time::Duration;

use super::{StarboardPeer, poll_until};

const HISTORY_WRITTEN_LINE: &str = "room history written";

/// Polls until one of `peers` logs that it wrote room history (`merge_and_write_history_records`),
/// the same log check the old harness made for `scenario history written`.
///
/// # Errors
/// Returns an error if no peer logs the line before `deadline`.
pub async fn wait_until_any_history_written(
    peers: &[&StarboardPeer],
    deadline: Duration,
) -> Result<(), String> {
    poll_until(
        "a holder wrote room history",
        deadline,
        async || {
            Ok::<bool, String>(
                peers
                    .iter()
                    .any(|peer| peer.log_contains(HISTORY_WRITTEN_LINE)),
            )
        },
        |written| *written,
    )
    .await
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::wait_until_any_history_written;

    #[tokio::test]
    async fn test_usage() {
        let peers: [&super::StarboardPeer; 0] = [];
        assert!(
            wait_until_any_history_written(&peers, Duration::from_millis(1))
                .await
                .is_err()
        );
    }
}
