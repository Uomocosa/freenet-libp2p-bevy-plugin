use crate::testing;

pub fn validate(args: &testing::NetworkArgs) -> Result<(), String> {
    let network = args.network;
    match network {
        testing::Network::Mainnet | testing::Network::DirectDial => Ok(()),
        testing::Network::LocalGateways => {
            Err("local-gateways network is not implemented yet".to_string())
        }
        testing::Network::Fake => args.orchestrator_port.map_or_else(
            || Err("fake network needs --orchestrator-port".to_string()),
            |_| Ok(()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::validate;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mainnet = testing::NetworkArgs {
            network: testing::Network::Mainnet,
            orchestrator_port: None,
        };
        assert_eq!(validate(&mainnet), Ok(()));
        let direct = testing::NetworkArgs {
            network: testing::Network::DirectDial,
            orchestrator_port: None,
        };
        assert_eq!(validate(&direct), Ok(()));
        let missing = testing::NetworkArgs {
            network: testing::Network::Fake,
            orchestrator_port: None,
        };
        assert!(validate(&missing).is_err());
        let local = testing::NetworkArgs {
            network: testing::Network::LocalGateways,
            orchestrator_port: None,
        };
        assert!(validate(&local).is_err());
    }
}
