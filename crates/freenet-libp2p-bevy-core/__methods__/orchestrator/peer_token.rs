use crate::testing;

pub fn peer_token(
    orchestrator: &testing::Orchestrator,
    peer_index: usize,
    default_token: &str,
) -> String {
    orchestrator
        .peers
        .get(peer_index)
        .and_then(|peer| peer.token_override.clone())
        .map_or_else(
            || default_token.to_string(),
            |override_token| override_token.replace(testing::ROOM_PLACEHOLDER, default_token),
        )
}

#[cfg(test)]
mod tests {
    use super::peer_token;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::Fake,
        );
        orchestrator.peer("painter").with_token("alpha");
        assert_eq!(peer_token(&orchestrator, 0, "default"), "alpha");
        assert_eq!(peer_token(&orchestrator, 3, "default"), "default");
    }
}
