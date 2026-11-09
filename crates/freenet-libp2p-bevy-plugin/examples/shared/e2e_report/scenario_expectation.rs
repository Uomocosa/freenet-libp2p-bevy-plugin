use std::collections::{BTreeMap, BTreeSet};

use freenet_libp2p_bevy_plugin::p2p;

use super::check_layer::CheckLayer;
use super::expectation_result::ExpectationResult;
use super::peer_run_result::PeerRunResult;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScenarioExpectation {
    pub canvas_converges: bool,
    pub min_members: usize,
    pub may_crash: BTreeSet<String>,
    pub join_must_fail: BTreeSet<String>,
}

impl ScenarioExpectation {
    #[must_use]
    pub fn check_peer_run_results(
        &self,
        results: &BTreeMap<String, Option<PeerRunResult>>,
    ) -> Vec<ExpectationResult> {
        let mut out = Vec::new();
        for (username, maybe_result) in results {
            let Some(result) = maybe_result else {
                out.push(ExpectationResult {
                    layer: CheckLayer::Scenario,
                    name: format!("{username} present"),
                    expected: "result.json present".to_string(),
                    actual: "crashed (result.json missing)".to_string(),
                    pass: self.may_crash.contains(username),
                });
                continue;
            };
            out.extend(self.scenario_rows(username, result));
            out.extend(Self::freenet_rows(username, result));
            out.extend(Self::libp2p_rows(username, result));
        }
        if let Some(row) = self.canvas_row(results) {
            out.push(row);
        }
        out
    }

    pub(crate) fn scenario_rows(
        &self,
        username: &str,
        result: &PeerRunResult,
    ) -> Vec<ExpectationResult> {
        let mut out = Vec::new();
        if self.join_must_fail.contains(username) {
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
            out.push(ExpectationResult {
                layer: CheckLayer::Scenario,
                name: format!("{username} join failed"),
                expected: "wait-join-failed step completed".to_string(),
                actual,
                pass: completed,
            });
            return out;
        }
        out.push(ExpectationResult {
            layer: CheckLayer::Scenario,
            name: format!("{username} members"),
            expected: format!("at least {} members seen", self.min_members),
            actual: format!("{} members seen", result.members_seen.len()),
            pass: result.members_seen.len() >= self.min_members,
        });
        if !self.may_crash.contains(username) {
            let actual = result.error.as_ref().map_or_else(
                || format!("exit {}", result.exit_code),
                |error| format!("exit {} error {error}", result.exit_code),
            );
            out.push(ExpectationResult {
                layer: CheckLayer::Scenario,
                name: format!("{username} exit"),
                expected: "exit 0 with no error".to_string(),
                actual,
                pass: result.exit_code == 0 && result.error.is_none(),
            });
        }
        out
    }

    pub(crate) fn freenet_rows(username: &str, result: &PeerRunResult) -> Vec<ExpectationResult> {
        let seen = result.freenet.freenet_peers_seen;
        let freenet_actual = result.freenet.lobby_fetched_after_secs.map_or_else(
            || format!("never fetched, {seen} freenet peers seen"),
            |secs| format!("fetched after {secs:.1}s with {seen} freenet peers"),
        );
        let row = ExpectationResult {
            layer: CheckLayer::Freenet,
            name: format!("{username} lobby fetched"),
            expected: "lobby fetched".to_string(),
            actual: freenet_actual,
            pass: result.freenet.lobby_fetched_after_secs.is_some(),
        };
        vec![row]
    }

    pub(crate) fn libp2p_rows(username: &str, result: &PeerRunResult) -> Vec<ExpectationResult> {
        let mut out = Vec::new();
        for (remote, kinds) in &result.connection_kinds {
            out.push(ExpectationResult {
                layer: CheckLayer::Libp2p,
                name: format!("{username} -> {} connection", remote.as_str()),
                expected: "connection kind observed".to_string(),
                actual: p2p::ConnectionKind::render_history(kinds),
                pass: true,
            });
        }
        out
    }

    pub(crate) fn canvas_row(
        &self,
        results: &BTreeMap<String, Option<PeerRunResult>>,
    ) -> Option<ExpectationResult> {
        if !self.canvas_converges {
            return None;
        }
        // Crashed peers (`may_crash`) leave a mid-paint snapshot behind; they
        // are not survivors, so convergence is over the rest.
        let surviving: Vec<(&String, &PeerRunResult)> = results
            .iter()
            .filter(|(username, _)| !self.may_crash.contains(username.as_str()))
            .filter_map(|(username, maybe)| maybe.as_ref().map(|result| (username, result)))
            .collect();
        let first = surviving.first().and_then(|(_, result)| {
            if result.tiles > 0 {
                result.canvas_hash
            } else {
                None
            }
        });
        let pass = first.is_some()
            && surviving
                .iter()
                .all(|(_, result)| result.tiles > 0 && result.canvas_hash == first);
        let actual = if surviving.is_empty() {
            "no surviving results".to_string()
        } else {
            surviving
                .iter()
                .map(|(username, result)| {
                    let hash = result
                        .canvas_hash
                        .map_or_else(|| "none".to_string(), |hash| hash.to_string());
                    format!("{username}: {hash} ({} tiles)", result.tiles)
                })
                .collect::<Vec<String>>()
                .join(", ")
        };
        let row = ExpectationResult {
            layer: CheckLayer::Scenario,
            name: "canvas converged".to_string(),
            expected: "same non-empty hash on all surviving peers".to_string(),
            actual,
            pass,
        };
        Some(row)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::super::peer_run_result::PeerRunResult;
    use super::ScenarioExpectation;
    use freenet_libp2p_bevy_plugin::{identity, p2p};

    use super::super::freenet_layer_result::FreenetLayerResult;

    fn expectation() -> ScenarioExpectation {
        ScenarioExpectation {
            canvas_converges: true,
            min_members: 1,
            may_crash: BTreeSet::new(),
            join_must_fail: BTreeSet::new(),
        }
    }

    fn result(username: &str) -> PeerRunResult {
        PeerRunResult {
            username: username.to_string(),
            canvas_hash: Some(42),
            tiles: 2,
            members_seen: std::iter::once(identity::PeerId::from("peer-a")).collect(),
            freenet: FreenetLayerResult {
                lobby_fetched_after_secs: Some(1.5),
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
        let results: BTreeMap<String, Option<PeerRunResult>> =
            std::iter::once(("p1".to_string(), Some(result("p1")))).collect();
        let rows = expectation().check_peer_run_results(&results);
        assert!(rows.iter().all(|row| row.pass));
        assert!(rows.iter().any(|row| row.name == "canvas converged"));
    }

    mod check_peer_run_results {
        use std::collections::{BTreeMap, BTreeSet};

        use super::super::super::freenet_layer_result::FreenetLayerResult;
        use super::super::super::peer_run_result::PeerRunResult;
        use super::super::ScenarioExpectation;
        use freenet_libp2p_bevy_plugin::{identity, p2p};

        fn expectation() -> ScenarioExpectation {
            ScenarioExpectation {
                canvas_converges: true,
                min_members: 1,
                may_crash: BTreeSet::new(),
                join_must_fail: BTreeSet::new(),
            }
        }

        fn result(username: &str) -> PeerRunResult {
            PeerRunResult {
                username: username.to_string(),
                canvas_hash: Some(42),
                tiles: 2,
                members_seen: std::iter::once(identity::PeerId::from("peer-a")).collect(),
                freenet: FreenetLayerResult {
                    lobby_fetched_after_secs: Some(1.5),
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
            let results: BTreeMap<String, Option<PeerRunResult>> =
                std::iter::once(("p1".to_string(), Some(result("p1")))).collect();
            let rows = expectation().check_peer_run_results(&results);
            assert!(rows.iter().all(|row| row.pass));
            assert!(rows.iter().any(|row| row.name == "canvas converged"));
        }

        #[test]
        fn a_missing_result_fails_unless_it_may_crash() {
            let results: BTreeMap<String, Option<PeerRunResult>> =
                std::iter::once(("p1".to_string(), None)).collect();
            let rows = expectation().check_peer_run_results(&results);
            assert!(
                !rows
                    .iter()
                    .find(|row| row.name == "p1 present")
                    .expect("row")
                    .pass
            );
            let mut may_crash = expectation();
            may_crash.may_crash.insert("p1".to_string());
            let rows = may_crash.check_peer_run_results(&results);
            assert!(
                rows.iter()
                    .find(|row| row.name == "p1 present")
                    .expect("row")
                    .pass
            );
        }

        #[test]
        fn diverging_canvases_fail_convergence() {
            let mut second = result("p2");
            second.canvas_hash = Some(99);
            let results: BTreeMap<String, Option<PeerRunResult>> = [
                ("p1".to_string(), Some(result("p1"))),
                ("p2".to_string(), Some(second)),
            ]
            .into_iter()
            .collect();
            let rows = expectation().check_peer_run_results(&results);
            assert!(
                !rows
                    .iter()
                    .find(|row| row.name == "canvas converged")
                    .expect("row")
                    .pass
            );
        }

        #[test]
        fn a_relayed_then_direct_connection_is_hole_punched() {
            let mut peer = result("p1");
            peer.connection_kinds.insert(
                identity::PeerId::from("peer-b"),
                vec![p2p::ConnectionKind::Relayed, p2p::ConnectionKind::Tcp],
            );
            let results: BTreeMap<String, Option<PeerRunResult>> =
                std::iter::once(("p1".to_string(), Some(peer))).collect();
            let rows = expectation().check_peer_run_results(&results);
            let row = rows
                .iter()
                .find(|row| row.name == "p1 -> peer-b connection")
                .expect("row");
            assert!(row.pass);
            assert!(row.actual.contains("hole-punched"));
        }
    }

    mod scenario_rows {
        use std::collections::BTreeSet;

        use super::super::super::freenet_layer_result::FreenetLayerResult;
        use super::super::super::peer_run_result::PeerRunResult;
        use super::super::ScenarioExpectation;
        use freenet_libp2p_bevy_plugin::testing::StepTiming;
        use freenet_libp2p_bevy_plugin::{identity, p2p};

        fn expectation() -> ScenarioExpectation {
            ScenarioExpectation {
                canvas_converges: true,
                min_members: 1,
                may_crash: BTreeSet::new(),
                join_must_fail: BTreeSet::new(),
            }
        }

        fn result(username: &str) -> PeerRunResult {
            PeerRunResult {
                username: username.to_string(),
                canvas_hash: Some(42),
                tiles: 2,
                members_seen: std::iter::once(identity::PeerId::from("peer-a")).collect(),
                freenet: FreenetLayerResult {
                    lobby_fetched_after_secs: Some(1.5),
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
            let rows = expectation().scenario_rows("p1", &result("p1"));
            assert!(rows.iter().all(|row| row.pass));
            assert!(rows.iter().any(|row| row.name == "p1 members"));
            assert!(rows.iter().any(|row| row.name == "p1 exit"));
        }

        #[test]
        fn a_join_that_must_fail_replaces_the_exit_row() {
            let mut must_fail = expectation();
            must_fail.join_must_fail.insert("p1".to_string());
            let mut failed = result("p1");
            failed.steps.push(StepTiming {
                raw: "wait-join-failed:room-1@420s".to_string(),
                secs: 3.0,
                failure: None,
            });
            let rows = must_fail.scenario_rows("p1", &failed);
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
            let rows = must_fail.scenario_rows("p1", &result("p1"));
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
            failed.steps.push(StepTiming {
                raw: "wait-join-failed:room-1@420s".to_string(),
                secs: 3.0,
                failure: None,
            });
            let rows = must_fail.scenario_rows("p1", &failed);
            assert!(!rows.iter().any(|row| row.name == "p1 members"));
            assert!(rows.iter().any(|row| row.name == "p1 join failed"));
        }
    }

    mod freenet_rows {
        use std::collections::BTreeSet;

        use super::super::super::freenet_layer_result::FreenetLayerResult;
        use super::super::super::peer_run_result::PeerRunResult;
        use super::super::ScenarioExpectation;
        use freenet_libp2p_bevy_plugin::{identity, p2p};

        fn expectation() -> ScenarioExpectation {
            ScenarioExpectation {
                canvas_converges: true,
                min_members: 1,
                may_crash: BTreeSet::new(),
                join_must_fail: BTreeSet::new(),
            }
        }

        fn result(username: &str) -> PeerRunResult {
            PeerRunResult {
                username: username.to_string(),
                canvas_hash: Some(42),
                tiles: 2,
                members_seen: std::iter::once(identity::PeerId::from("peer-a")).collect(),
                freenet: FreenetLayerResult {
                    lobby_fetched_after_secs: Some(1.5),
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
            let rows = ScenarioExpectation::freenet_rows("p1", &result("p1"));
            assert_eq!(rows.len(), 1);
            assert!(rows.iter().all(|row| row.pass));
            assert_eq!(rows[0].name, "p1 lobby fetched");
        }
    }

    mod libp2p_rows {
        use std::collections::BTreeSet;

        use super::super::super::freenet_layer_result::FreenetLayerResult;
        use super::super::super::peer_run_result::PeerRunResult;
        use super::super::ScenarioExpectation;
        use freenet_libp2p_bevy_plugin::{identity, p2p};

        fn expectation() -> ScenarioExpectation {
            ScenarioExpectation {
                canvas_converges: true,
                min_members: 1,
                may_crash: BTreeSet::new(),
                join_must_fail: BTreeSet::new(),
            }
        }

        fn result(username: &str) -> PeerRunResult {
            PeerRunResult {
                username: username.to_string(),
                canvas_hash: Some(42),
                tiles: 2,
                members_seen: std::iter::once(identity::PeerId::from("peer-a")).collect(),
                freenet: FreenetLayerResult {
                    lobby_fetched_after_secs: Some(1.5),
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
            let rows = ScenarioExpectation::libp2p_rows("p1", &result("p1"));
            assert_eq!(rows.len(), 1);
            assert!(rows.iter().all(|row| row.pass));
            assert_eq!(rows[0].name, "p1 -> peer-a connection");
        }
    }

    mod canvas_row {
        use std::collections::{BTreeMap, BTreeSet};

        use super::super::super::freenet_layer_result::FreenetLayerResult;
        use super::super::super::peer_run_result::PeerRunResult;
        use super::super::ScenarioExpectation;
        use freenet_libp2p_bevy_plugin::{identity, p2p};

        fn expectation() -> ScenarioExpectation {
            ScenarioExpectation {
                canvas_converges: true,
                min_members: 1,
                may_crash: BTreeSet::new(),
                join_must_fail: BTreeSet::new(),
            }
        }

        fn result(username: &str) -> PeerRunResult {
            PeerRunResult {
                username: username.to_string(),
                canvas_hash: Some(42),
                tiles: 2,
                members_seen: std::iter::once(identity::PeerId::from("peer-a")).collect(),
                freenet: FreenetLayerResult {
                    lobby_fetched_after_secs: Some(1.5),
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
            let results: BTreeMap<String, Option<PeerRunResult>> =
                std::iter::once(("p1".to_string(), Some(result("p1")))).collect();
            let row = expectation().canvas_row(&results).expect("row");
            assert!(row.pass);
            assert_eq!(row.name, "canvas converged");
        }

        #[test]
        fn disabled_convergence_returns_no_row() {
            let mut no_canvas = expectation();
            no_canvas.canvas_converges = false;
            let results: BTreeMap<String, Option<PeerRunResult>> =
                std::iter::once(("p1".to_string(), Some(result("p1")))).collect();
            assert!(no_canvas.canvas_row(&results).is_none());
        }

        /// A crashed peer's mid-paint snapshot is not a survivor: survivors
        /// must still agree, and the crashed canvas must not fail the row.
        #[test]
        fn crashed_peers_are_not_survivors() {
            let mut crashed = expectation();
            crashed.may_crash = std::iter::once("p3".to_string()).collect();
            let mut diverged = result("p3");
            diverged.canvas_hash = Some(7);
            diverged.tiles = 1;
            let results: BTreeMap<String, Option<PeerRunResult>> = [
                ("p1".to_string(), Some(result("p1"))),
                ("p2".to_string(), Some(result("p2"))),
                ("p3".to_string(), Some(diverged)),
            ]
            .into_iter()
            .collect();
            let row = crashed.canvas_row(&results).expect("row");
            assert!(row.pass, "survivors agree: {}", row.actual);
            assert!(
                !row.actual.contains("p3"),
                "crashed peer shown: {}",
                row.actual
            );
        }
    }
}
