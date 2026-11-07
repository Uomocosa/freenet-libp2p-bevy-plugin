use crate::testing;

#[must_use]
pub fn runs_here(orchestrator: &testing::Orchestrator, index: usize) -> bool {
    if orchestrator.placement.is_empty() {
        return true;
    }
    let Some(peer) = orchestrator.peers.get(index) else {
        return false;
    };
    testing::Orchestrator::peer_runs_here(
        &peer.name,
        &orchestrator.machine,
        &orchestrator.placement,
    )
}

#[cfg(test)]
mod tests {
    use super::runs_here;
    use crate::testing;

    fn orchestrator() -> testing::Orchestrator {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        );
        orchestrator.peer("alice");
        orchestrator.peer("bob");
        orchestrator
    }

    fn placed() -> testing::Orchestrator {
        let mut orchestrator = orchestrator();
        orchestrator.machine = "a".to_string();
        orchestrator.placement = vec![
            ("alice".to_string(), "a".to_string()),
            ("bob".to_string(), "b".to_string()),
        ];
        orchestrator
    }

    #[test]
    fn test_usage() {
        let orchestrator = orchestrator();
        assert!(runs_here(&orchestrator, 0));
        assert!(runs_here(&orchestrator, 1));
        assert!(runs_here(&orchestrator, 7));
    }

    #[test]
    fn placement_keeps_only_this_machines_peers() {
        let orchestrator = placed();
        assert!(runs_here(&orchestrator, 0));
        assert!(!runs_here(&orchestrator, 1));
        assert!(!runs_here(&orchestrator, 7));
    }
}
