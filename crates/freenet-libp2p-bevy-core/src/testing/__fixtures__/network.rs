use atomic_delegate_macros::atomic_delegates;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Network {
    Mainnet,
    LocalGateways,
    DirectDial,
    Fake,
}

#[atomic_delegates]
impl Network {
    #[must_use]
    pub fn from_env() -> Self {}
}

impl Network {
    pub fn require_real(&self, scenario: &str) -> Result<(), String> {
        if *self == Self::Fake {
            Err(format!(
                "{scenario} needs a real network: on fake the example reports ring_peers=1 without a node, so wait-ring-joined proves nothing; run with E2E_NETWORK=mainnet"
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Network;
    use clap::ValueEnum;

    #[test]
    fn test_usage() {
        assert_eq!(
            Network::from_str("mainnet", false).expect("mainnet"),
            Network::Mainnet
        );
        assert_eq!(
            Network::from_str("direct-dial", false).expect("direct-dial"),
            Network::DirectDial
        );
        assert_eq!(
            Network::from_str("fake", false).expect("fake"),
            Network::Fake
        );
        if Network::from_env() == Network::Fake {
            assert!(Network::from_env().require_real("probe").is_err());
        } else {
            assert!(Network::from_env().require_real("probe").is_ok());
        }
        assert!(Network::Mainnet.require_real("probe").is_ok());
    }
}
