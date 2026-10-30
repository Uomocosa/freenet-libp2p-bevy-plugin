use crate::e2e_report;
use crate::p2p;

#[must_use]
pub fn libp2p_rows(
    _expectation: &e2e_report::ScenarioExpectation,
    username: &str,
    result: &e2e_report::PeerRunResult,
) -> Vec<e2e_report::ExpectationResult> {
    let mut out = Vec::new();
    for (remote, kinds) in &result.connection_kinds {
        out.push(e2e_report::ExpectationResult {
            layer: e2e_report::CheckLayer::Libp2p,
            name: format!("{username} -> {} connection", remote.as_str()),
            expected: "connection kind observed".to_string(),
            actual: p2p::ConnectionKind::render_history(kinds),
            pass: true,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::libp2p_rows;
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
        let rows = libp2p_rows(&expectation(), "p1", &result("p1"));
        assert_eq!(rows.len(), 1);
        assert!(rows.iter().all(|row| row.pass));
        assert_eq!(rows[0].name, "p1 -> peer-a connection");
    }
}
