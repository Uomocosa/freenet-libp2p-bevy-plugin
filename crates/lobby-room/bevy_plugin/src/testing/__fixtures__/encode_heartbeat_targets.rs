use crate::discovery;

pub fn encode_heartbeat_targets(
    targets: Vec<discovery::state_machine::HeartbeatTarget>,
) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(&discovery::LobbyWire::HeartbeatTargets(targets))
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::testing;

    use super::encode_heartbeat_targets;

    #[test]
    fn test_usage() {
        let target = testing::heartbeat_target("room-a", 10, discovery::RoomMembership::Occupant);
        let bytes = encode_heartbeat_targets(vec![target]).expect("encode");
        let wire: discovery::LobbyWire = bincode::deserialize(&bytes).expect("decode");
        assert!(matches!(
            wire,
            discovery::LobbyWire::HeartbeatTargets(targets) if targets.len() == 1
        ));
    }
}
