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
    use super::NetworkArgs;
    use clap::Parser;

    #[derive(Debug, Parser)]
    struct Probe {
        #[command(flatten)]
        network: NetworkArgs,
    }

    #[test]
    fn test_usage() {
        let probe = Probe::try_parse_from(["probe"]).expect("defaults");
        assert!(matches!(
            probe.network.network,
            crate::testing::Network::Mainnet
        ));
        assert_eq!(probe.network.orchestrator_port, None);
        let probe =
            Probe::try_parse_from(["probe", "--network", "fake", "--orchestrator-port", "4001"])
                .expect("fake");
        assert!(matches!(
            probe.network.network,
            crate::testing::Network::Fake
        ));
        assert_eq!(probe.network.orchestrator_port, Some(4001));
    }
}
