use crate::testing;

#[must_use]
pub fn is_remote_peer(orchestrator: &testing::Orchestrator, name: &str) -> bool {
    if orchestrator.placement.is_empty() {
        return false;
    }
    !testing::Orchestrator::peer_runs_here(name, &orchestrator.machine, &orchestrator.placement)
}

#[cfg(test)]
mod tests {
    use super::is_remote_peer;
    use crate::testing;

    fn placed() -> testing::Orchestrator {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        );
        orchestrator.peer("alice");
        orchestrator.peer("bob");
        orchestrator.machine = "a".to_string();
        orchestrator.placement = vec![
            ("alice".to_string(), "a".to_string()),
            ("bob".to_string(), "b".to_string()),
        ];
        orchestrator
    }

    #[test]
    fn test_usage() {
        let orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        );
        assert!(!is_remote_peer(&orchestrator, "bob"));
    }

    #[test]
    fn placed_elsewhere_is_remote() {
        let orchestrator = placed();
        assert!(!is_remote_peer(&orchestrator, "alice"));
        assert!(is_remote_peer(&orchestrator, "bob"));
        assert!(is_remote_peer(&orchestrator, "mallory"));
    }
}
