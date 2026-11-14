use std::path::Path;
use std::time::Duration;

use super::{LobbyRoomPeer, poll_until};
use crate::common::marker_times;

const MESH_MARKER: &str = "lobby mesh ";
const CONVERGE_BUDGET_MS: u64 = 500;

/// The first time at or after `after_ms` (day millis) that the app logged a mesh line with
/// `connected=<connected>`, or `None` while the mesh is not yet complete.
#[must_use]
pub fn fully_meshed_at(log: &Path, after_ms: u64, connected: usize) -> Option<u64> {
    let want = format!("connected={connected}");
    marker_times(log, MESH_MARKER)
        .into_iter()
        .find(|(ms, line)| *ms >= after_ms && line.split_whitespace().any(|token| token == want))
        .map(|(ms, _)| ms)
}

/// Waits until every member logged a full mesh of `members` peers after `last_press`, then checks
/// that the last press reached the full mesh within the 500 ms budget of the old `lobby_room_join`
/// (`did_mesh_converge_within_budget`, `tests/e2e/tests/common/scenario.rs`).
///
/// # Errors
/// Returns an error if the mesh never completes before `deadline`, or if it takes longer than the budget.
pub async fn expect_meshed_within_budget(
    members: &[LobbyRoomPeer],
    last_press: u64,
    deadline: Duration,
) -> Result<String, String> {
    let connected = members.len().saturating_sub(1);
    let found = poll_until(
        "full mesh",
        deadline,
        async || Ok(meshed_times(members, last_press, connected)),
        Option::is_some,
    )
    .await?;
    let Some(times) = found else {
        return Err("never fully meshed after the last press".to_string());
    };
    let done = times.into_iter().max().unwrap_or(last_press);
    let converge_ms = done.saturating_sub(last_press);
    let detail =
        format!("converge {converge_ms}ms budget={CONVERGE_BUDGET_MS}ms (last press -> full mesh)");
    if converge_ms <= CONVERGE_BUDGET_MS {
        Ok(detail)
    } else {
        Err(detail)
    }
}

fn meshed_times(members: &[LobbyRoomPeer], last_press: u64, connected: usize) -> Option<Vec<u64>> {
    members
        .iter()
        .map(|member| fully_meshed_at(member.log_path(), last_press, connected))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::fully_meshed_at;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("temp dir");
        let log = dir.path().join("peer.log");
        let mut file = std::fs::File::create(&log).expect("log");
        writeln!(
            file,
            "2026-11-14T10:00:00.100Z lobby mesh room=r connected=1"
        )
        .expect("write");
        writeln!(
            file,
            "2026-11-14T10:00:02.000Z lobby mesh room=r connected=2"
        )
        .expect("write");
        let at_ten = 10 * 3_600_000;
        assert_eq!(fully_meshed_at(&log, at_ten, 2), Some(at_ten + 2_000));
        assert_eq!(fully_meshed_at(&log, at_ten + 3_000, 2), None);
        assert_eq!(fully_meshed_at(&log, at_ten, 3), None);
    }
}
