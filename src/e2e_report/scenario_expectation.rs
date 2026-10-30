use std::collections::BTreeSet;

use atomic_delegate_macros::atomic_delegates;

use crate::e2e_report;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScenarioExpectation {
    pub canvas_converges: bool,
    pub min_members: usize,
    pub may_crash: BTreeSet<String>,
    pub join_must_fail: BTreeSet<String>,
}

#[atomic_delegates]
impl ScenarioExpectation {
    pub fn check_peer_run_results(
        &self,
        results: &std::collections::BTreeMap<String, Option<e2e_report::PeerRunResult>>,
    ) -> Vec<e2e_report::ExpectationResult> {
    }
}

#[atomic_delegates]
impl ScenarioExpectation {
    pub(crate) fn scenario_rows(
        &self,
        username: &str,
        result: &e2e_report::PeerRunResult,
    ) -> Vec<e2e_report::ExpectationResult> {
    }

    pub(crate) fn freenet_rows(
        &self,
        username: &str,
        result: &e2e_report::PeerRunResult,
    ) -> Vec<e2e_report::ExpectationResult> {
    }

    pub(crate) fn libp2p_rows(
        &self,
        username: &str,
        result: &e2e_report::PeerRunResult,
    ) -> Vec<e2e_report::ExpectationResult> {
    }

    pub(crate) fn canvas_row(
        &self,
        results: &std::collections::BTreeMap<String, Option<e2e_report::PeerRunResult>>,
    ) -> Option<e2e_report::ExpectationResult> {
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::ScenarioExpectation;
    use crate::e2e_report;
    use crate::identity;
    use crate::p2p;

    fn expectation() -> ScenarioExpectation {
        ScenarioExpectation {
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
        let results: BTreeMap<String, Option<e2e_report::PeerRunResult>> =
            std::iter::once(("p1".to_string(), Some(result("p1")))).collect();
        let rows = expectation().check_peer_run_results(&results);
        assert!(rows.iter().all(|row| row.pass));
        assert!(rows.iter().any(|row| row.name == "canvas converged"));
    }
}
