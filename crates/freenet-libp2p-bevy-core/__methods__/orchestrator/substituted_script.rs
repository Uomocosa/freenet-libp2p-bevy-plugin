use crate::testing;

#[must_use]
pub fn substituted_script(
    orchestrator: &testing::Orchestrator,
    peer_index: usize,
    room: &str,
    token: &str,
    live: &[testing::LivePeer],
) -> Option<String> {
    let base = orchestrator.peer_script(peer_index, room).ok()?;
    let mut rendered = Vec::new();
    for fragment in base.split(", ") {
        let substituted = orchestrator.substitute_fragment(fragment, token, live)?;
        rendered.push(substituted);
    }
    Some(rendered.join(", "))
}

#[cfg(test)]
mod tests {
    use super::substituted_script;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        );
        orchestrator.peer("alice");
        let script =
            substituted_script(&orchestrator, 0, "room-1", "token", &[]).expect("empty script");
        assert_eq!(script, "");
        assert_eq!(
            substituted_script(&orchestrator, 7, "room-1", "token", &[]),
            None
        );
    }
}
