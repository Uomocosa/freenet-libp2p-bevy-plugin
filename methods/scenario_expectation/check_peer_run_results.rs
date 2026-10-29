use std::collections::BTreeMap;

use crate::e2e_report;
use crate::p2p;

#[must_use]
pub fn check_peer_run_results(
    expectation: &e2e_report::ScenarioExpectation,
    results: &BTreeMap<String, Option<e2e_report::PeerRunResult>>,
) -> Vec<e2e_report::ExpectationResult> {
    let mut out = Vec::new();
    for (username, maybe_result) in results {
        let Some(result) = maybe_result else {
            out.push(e2e_report::ExpectationResult {
                layer: e2e_report::CheckLayer::Scenario,
                name: format!("{username} present"),
                expected: "result.json present".to_string(),
                actual: "crashed (result.json missing)".to_string(),
                pass: expectation.may_crash.contains(username),
            });
            continue;
        };
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
        let seen = result.freenet.freenet_peers_seen;
        let freenet_actual = result.freenet.lobby_attached_after_secs.map_or_else(
            || format!("never attached, {seen} freenet peers seen"),
            |secs| format!("attached after {secs:.1}s with {seen} freenet peers"),
        );
        out.push(e2e_report::ExpectationResult {
            layer: e2e_report::CheckLayer::Freenet,
            name: format!("{username} lobby attached"),
            expected: "lobby attached".to_string(),
            actual: freenet_actual,
            pass: result.freenet.lobby_attached_after_secs.is_some(),
        });
        for (remote, kinds) in &result.connection_kinds {
            out.push(e2e_report::ExpectationResult {
                layer: e2e_report::CheckLayer::Libp2p,
                name: format!("{username} -> {} connection", remote.as_str()),
                expected: "connection kind observed".to_string(),
                actual: p2p::ConnectionKind::render_history(kinds),
                pass: true,
            });
        }
    }
    if expectation.canvas_converges {
        let surviving: Vec<(&String, &e2e_report::PeerRunResult)> = results
            .iter()
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
        out.push(e2e_report::ExpectationResult {
            layer: e2e_report::CheckLayer::Scenario,
            name: "canvas converged".to_string(),
            expected: "same non-empty hash on all surviving peers".to_string(),
            actual,
            pass,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::check_peer_run_results;
    use crate::e2e_report;
    use crate::identity;
    use crate::p2p;

    fn expectation() -> e2e_report::ScenarioExpectation {
        e2e_report::ScenarioExpectation {
            canvas_converges: true,
            min_members: 1,
            may_crash: BTreeSet::new(),
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
        let rows = check_peer_run_results(&expectation(), &results);
        assert!(rows.iter().all(|row| row.pass));
        assert!(rows.iter().any(|row| row.name == "canvas converged"));
    }

    #[test]
    fn a_missing_result_fails_unless_it_may_crash() {
        let results: BTreeMap<String, Option<e2e_report::PeerRunResult>> =
            std::iter::once(("p1".to_string(), None)).collect();
        let rows = check_peer_run_results(&expectation(), &results);
        assert!(
            !rows
                .iter()
                .find(|row| row.name == "p1 present")
                .expect("row")
                .pass
        );
        let mut may_crash = expectation();
        may_crash.may_crash.insert("p1".to_string());
        let rows = check_peer_run_results(&may_crash, &results);
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
        let results: BTreeMap<String, Option<e2e_report::PeerRunResult>> = [
            ("p1".to_string(), Some(result("p1"))),
            ("p2".to_string(), Some(second)),
        ]
        .into_iter()
        .collect();
        let rows = check_peer_run_results(&expectation(), &results);
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
        let results: BTreeMap<String, Option<e2e_report::PeerRunResult>> =
            std::iter::once(("p1".to_string(), Some(peer))).collect();
        let rows = check_peer_run_results(&expectation(), &results);
        let row = rows
            .iter()
            .find(|row| row.name == "p1 -> peer-b connection")
            .expect("row");
        assert!(row.pass);
        assert!(row.actual.contains("hole-punched"));
    }
}
