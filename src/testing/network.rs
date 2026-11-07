use atomic_delegate_macros::atomic_delegates;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Network {
    Mainnet,
    LocalGateways,
    Fake,
}

#[atomic_delegates]
impl Network {
    #[must_use]
    pub fn from_env() -> Self {}
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
            Network::from_str("fake", false).expect("fake"),
            Network::Fake
        );
    }
}
