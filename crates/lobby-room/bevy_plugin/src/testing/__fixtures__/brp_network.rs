use std::path::PathBuf;

use freenet_libp2p_bevy_core::testing::harness as core_harness;

use crate::testing;

#[must_use]
pub fn brp_network(
    network: testing::Network,
    token: &str,
    run_dir: &str,
) -> testing::harness::BrpNetwork {
    testing::harness::BrpNetwork {
        network,
        token: token.to_string(),
        deadlines: core_harness::Deadlines(1.0),
        run_dir: PathBuf::from(run_dir),
        display: None,
        orchestrator: None,
        _helper_nodes: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use crate::testing;

    use super::brp_network;

    #[test]
    fn test_usage() {
        let net = brp_network(testing::Network::Fake, "brp-1", "/tmp/brp-test");
        assert_eq!(net.network_arg(), "fake");
        assert_eq!(net.token, "brp-1");
    }
}
