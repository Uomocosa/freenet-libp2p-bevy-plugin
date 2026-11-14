use freenet_libp2p_bevy_plugin::testing::Network;

/// Refuses the fake network for a scenario whose ring check is only meaningful on a real node.
/// The fake run of the example sets its ring count to 1 without a node (`examples/lobby_room/src/main.rs`).
///
/// # Errors
/// Returns an error when `E2E_NETWORK` selects the fake network.
pub fn require_real_network(scenario: &str) -> Result<(), String> {
    if Network::from_env() == Network::Fake {
        Err(format!(
            "{scenario} needs a real network: on fake the example reports ring_peers=1 without a node, so wait-ring-joined proves nothing; run with E2E_NETWORK=mainnet"
        ))
    } else {
        Ok(())
    }
}
