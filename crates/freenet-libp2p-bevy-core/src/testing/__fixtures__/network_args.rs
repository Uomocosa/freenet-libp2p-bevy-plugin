use atomic_delegate_macros::atomic_delegates;
use clap::Args;

use crate::testing;

#[derive(Debug, Clone, Args)]
pub struct NetworkArgs {
    #[arg(long, value_enum, default_value = "mainnet")]
    pub network: testing::Network,
    #[arg(long)]
    pub orchestrator_port: Option<u16>,
}

#[atomic_delegates]
impl NetworkArgs {
    pub fn validate(&self) -> Result<(), String> {}
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::testing;

    #[test]
    fn test_usage() {
        let args = testing::NetworkArgs {
            network: testing::Network::Fake,
            orchestrator_port: Some(4001),
        };
        assert!(args.validate().is_ok());
        assert!(matches!(args.network, testing::Network::Fake));
        assert_eq!(args.orchestrator_port, Some(4001));
    }

    #[test]
    fn probe_parses_the_network_flags() {
        let probe = testing::NetworkArgsProbe::try_parse_from(["probe"]).expect("defaults");
        assert!(matches!(probe.network.network, testing::Network::Mainnet));
        assert_eq!(probe.network.orchestrator_port, None);
        let probe = testing::NetworkArgsProbe::try_parse_from([
            "probe",
            "--network",
            "fake",
            "--orchestrator-port",
            "4001",
        ])
        .expect("fake");
        assert!(matches!(probe.network.network, testing::Network::Fake));
        assert_eq!(probe.network.orchestrator_port, Some(4001));
    }
}
