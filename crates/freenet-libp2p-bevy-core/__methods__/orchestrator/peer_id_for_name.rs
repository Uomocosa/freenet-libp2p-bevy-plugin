use crate::identity;
use crate::testing;

pub fn peer_id_for_name(
    orchestrator: &testing::Orchestrator,
    token: &str,
    name: &str,
) -> Result<identity::PeerId, String> {
    let path = orchestrator.peer_identity_file(token, name);
    let saved = identity::SavedKeypair::try_from(path.as_path())
        .map_err(|error| format!("identity file for {name}: {error}"))?;
    Ok(identity::PeerId::from(
        saved.public().to_peer_id().to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::peer_id_for_name;
    use crate::testing;

    #[test]
    fn test_usage() {
        let orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        );
        let first = peer_id_for_name(&orchestrator, "probe-token", "probe-peer").expect("identity");
        let second =
            peer_id_for_name(&orchestrator, "probe-token", "probe-peer").expect("identity");
        assert_eq!(first, second);
        assert_ne!(first.as_str(), "");
        let path = orchestrator.peer_identity_file("probe-token", "probe-peer");
        assert!(path.starts_with(std::env::temp_dir()));
        std::fs::remove_file(&path).expect("cleanup");
    }
}
