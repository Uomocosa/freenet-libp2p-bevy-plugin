use std::collections::BTreeMap;

use crate::e2e_report;

#[must_use]
pub fn canvas_row(
    expectation: &e2e_report::ScenarioExpectation,
    results: &BTreeMap<String, Option<e2e_report::PeerRunResult>>,
) -> Option<e2e_report::ExpectationResult> {
    if !expectation.canvas_converges {
        return None;
    }
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
    let row = e2e_report::ExpectationResult {
        layer: e2e_report::CheckLayer::Scenario,
        name: "canvas converged".to_string(),
        expected: "same non-empty hash on all surviving peers".to_string(),
        actual,
        pass,
    };
    Some(row)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::canvas_row;
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
        let results: BTreeMap<String, Option<e2e_report::PeerRunResult>> =
            std::iter::once(("p1".to_string(), Some(result("p1")))).collect();
        let row = canvas_row(&expectation(), &results).expect("row");
        assert!(row.pass);
        assert_eq!(row.name, "canvas converged");
    }

    #[test]
    fn disabled_convergence_returns_no_row() {
        let mut no_canvas = expectation();
        no_canvas.canvas_converges = false;
        let results: BTreeMap<String, Option<e2e_report::PeerRunResult>> =
            std::iter::once(("p1".to_string(), Some(result("p1")))).collect();
        assert!(canvas_row(&no_canvas, &results).is_none());
    }
}
