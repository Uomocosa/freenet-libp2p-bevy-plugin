use freenet_libp2p_bevy_plugin::testing::{Network, NetworkArgs};

/// Turns the `--network` choice into the node and plugin args, refusing the networks that only
/// the e2e apps (`tests/e2e_apps`) can drive.
pub fn network_args(network: Network) -> Result<NetworkArgs, String> {
    match network {
        Network::Fake => {
            Err("the fake network is only available in the e2e apps (tests/e2e_apps)".to_string())
        }
        Network::DirectDial => Err(
            "the direct-dial network is only available in the e2e apps (tests/e2e_apps)"
                .to_string(),
        ),
        Network::Mainnet | Network::LocalGateways => Ok(NetworkArgs {
            network,
            orchestrator_port: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::testing::Network;

    use super::network_args;

    #[test]
    fn test_usage() {
        assert!(network_args(Network::Mainnet).is_ok());
        assert!(network_args(Network::LocalGateways).is_ok());
        assert!(network_args(Network::DirectDial).is_err());
        let error = network_args(Network::Fake).expect_err("fake is refused");
        assert!(error.contains("only available in the e2e apps"));
    }
}
