use crate::testing;

#[must_use]
pub fn from_env() -> testing::Network {
    let raw = std::env::var("E2E_NETWORK").unwrap_or_default();
    let trimmed = raw.trim().to_ascii_lowercase();
    match trimmed.as_str() {
        "mainnet" => testing::Network::Mainnet,
        "local-gateways" | "local_gateways" | "local" => testing::Network::LocalGateways,
        _ => testing::Network::Fake,
    }
}

#[cfg(test)]
mod tests {
    use super::from_env;
    use crate::testing;

    #[test]
    fn test_usage() {
        let network = testing::Network::from_env();
        assert!(matches!(
            network,
            testing::Network::Fake | testing::Network::Mainnet | testing::Network::LocalGateways
        ));
        assert_eq!(from_env(), testing::Network::from_env());
    }
}
