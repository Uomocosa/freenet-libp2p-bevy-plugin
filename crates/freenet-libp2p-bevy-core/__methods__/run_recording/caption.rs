use std::time::Duration;

use crate::testing;

pub fn caption(
    test_name: &str,
    outcome: testing::harness::RecordingOutcome,
    elapsed: Duration,
) -> String {
    format!(
        "{test_name} {outcome} — {seconds:.1} s",
        seconds = elapsed.as_secs_f64()
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::caption;
    use crate::testing;

    #[test]
    fn test_usage() {
        let passed = caption(
            "lobby_room_two_peers_meet",
            testing::harness::RecordingOutcome::Passed,
            Duration::from_millis(42_300),
        );
        assert_eq!(passed, "lobby_room_two_peers_meet PASSED — 42.3 s");
        let failed = caption(
            "one_peer_paints",
            testing::harness::RecordingOutcome::Failed,
            Duration::from_secs(3),
        );
        assert_eq!(failed, "one_peer_paints FAILED — 3.0 s");
    }
}
