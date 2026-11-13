use crate::testing;

pub fn local_peer_names(orchestrator: &testing::Orchestrator) -> Vec<String> {
    orchestrator
        .peers
        .iter()
        .filter(|peer| {
            testing::Orchestrator::peer_runs_here(
                &peer.name,
                &orchestrator.machine,
                &orchestrator.placement,
            )
        })
        .map(|peer| peer.name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::local_peer_names;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::Fake,
        );
        orchestrator.peer("alice");
        orchestrator.peer("bob");
        assert_eq!(
            local_peer_names(&orchestrator),
            vec!["alice".to_string(), "bob".to_string()]
        );
        orchestrator.machine = "a".to_string();
        orchestrator.placement = vec![("bob".to_string(), "b".to_string())];
        assert_eq!(local_peer_names(&orchestrator), Vec::<String>::new());
    }
}
