use crate::e2e_report;

#[must_use]
pub fn scenario_rows(
    expectation: &e2e_report::ScenarioExpectation,
    username: &str,
    result: &e2e_report::PeerRunResult,
) -> Vec<e2e_report::ExpectationResult> {
    let mut out = Vec::new();
    if expectation.join_must_fail.contains(username) {
        let completed = result
            .steps
            .iter()
            .any(|step| step.raw.starts_with("wait-join-failed") && step.failure.is_none());
        let actual = result
            .steps
            .iter()
            .find(|step| step.raw.starts_with("wait-join-failed"))
            .map_or_else(
                || "no wait-join-failed step".to_string(),
                |step| step.raw.clone(),
            );
        out.push(e2e_report::ExpectationResult {
            layer: e2e_report::CheckLayer::Scenario,
            name: format!("{username} join failed"),
            expected: "wait-join-failed step completed".to_string(),
            actual,
            pass: completed,
        });
        return out;
    }
    out.push(e2e_report::ExpectationResult {
        layer: e2e_report::CheckLayer::Scenario,
        name: format!("{username} members"),
        expected: format!("at least {} members seen", expectation.min_members),
        actual: format!("{} members seen", result.members_seen.len()),
        pass: result.members_seen.len() >= expectation.min_members,
    });
    if !expectation.may_crash.contains(username) {
        let actual = result.error.as_ref().map_or_else(
            || format!("exit {}", result.exit_code),
            |error| format!("exit {} error {error}", result.exit_code),
        );
        out.push(e2e_report::ExpectationResult {
            layer: e2e_report::CheckLayer::Scenario,
            name: format!("{username} exit"),
            expected: "exit 0 with no error".to_string(),
            actual,
            pass: result.exit_code == 0 && result.error.is_none(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::scenario_rows;
    use crate::e2e_report;
    use crate::identity;
    use crate::p2p;

    fn expectation() -> e2e_report::ScenarioExpectation {
        e2e_report::ScenarioExpectation {
            canvas_converges: true,
            min_members: 1,
            may_crash: BTreeSet::new(),
            join_must_fail: BTreeSet::new(),
        }
    }

    fn result(username: &str) -> e2e_report::PeerRunResult {
        e2e_report::PeerRunResult {
            username: username.to_string(),
            canvas_hash: Some(42),
            tiles: 2,
            members_seen: std::iter::once(identity::PeerId::from("peer-a")).collect(),
            freenet: e2e_report::FreenetLayerResult {
                lobby_attached_after_secs: Some(1.5),
                freenet_peers_seen: 2,
            },
            connection_kinds: std::iter::once((
                identity::PeerId::from("peer-a"),
                vec![p2p::ConnectionKind::Tcp],
            ))
            .collect(),
            steps: Vec::new(),
            error: None,
            exit_code: 0,
        }
    }

    #[test]
    fn test_usage() {
        let rows = scenario_rows(&expectation(), "p1", &result("p1"));
        assert!(rows.iter().all(|row| row.pass));
        assert!(rows.iter().any(|row| row.name == "p1 members"));
        assert!(rows.iter().any(|row| row.name == "p1 exit"));
    }

    #[test]
    fn a_join_that_must_fail_replaces_the_exit_row() {
        let mut must_fail = expectation();
        must_fail.join_must_fail.insert("p1".to_string());
        let mut failed = result("p1");
        failed.steps.push(e2e_report::StepTiming {
            raw: "wait-join-failed:room-1@420s".to_string(),
            secs: 3.0,
            failure: None,
        });
        let rows = scenario_rows(&must_fail, "p1", &failed);
        assert!(
            rows.iter()
                .any(|row| row.name == "p1 join failed" && row.pass)
        );
        assert!(!rows.iter().any(|row| row.name == "p1 exit"));
    }

    #[test]
    fn a_missing_join_failure_step_fails() {
        let mut must_fail = expectation();
        must_fail.join_must_fail.insert("p1".to_string());
        let rows = scenario_rows(&must_fail, "p1", &result("p1"));
        let row = rows
            .iter()
            .find(|row| row.name == "p1 join failed")
            .expect("row");
        assert!(!row.pass);
    }

    #[test]
    fn a_peer_that_must_fail_join_has_no_members_row() {
        let mut must_fail = expectation();
        must_fail.join_must_fail.insert("p1".to_string());
        let mut failed = result("p1");
        failed.steps.push(e2e_report::StepTiming {
            raw: "wait-join-failed:room-1@420s".to_string(),
            secs: 3.0,
            failure: None,
        });
        let rows = scenario_rows(&must_fail, "p1", &failed);
        assert!(!rows.iter().any(|row| row.name == "p1 members"));
        assert!(rows.iter().any(|row| row.name == "p1 join failed"));
    }
}
