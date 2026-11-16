use std::path::Path;

use freenet_libp2p_bevy_core::testing::harness as core_harness;

use crate::testing;
use crate::time;

pub async fn try_from_network_setup(
    network: testing::Network,
    setup: testing::harness::LobbySetup,
) -> Result<testing::harness::BrpNetwork, String> {
    let stamp = time::UnixTime::now().as_secs();
    let run_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(".local-run")
        .join(format!("brp-{stamp}"));
    std::fs::create_dir_all(&run_dir).map_err(|e| format!("run dir: {e}"))?;
    let deadlines = core_harness::Deadlines::from(network);
    match network {
        testing::Network::Fake => {
            let server =
                testing::FakeNetworkServer::start_with(Box::new(testing::LobbyKit::default()))
                    .await?;
            Ok(testing::harness::BrpNetwork {
                network,
                token: format!("brp-{}", core_harness::UnixNanos::now()),
                deadlines,
                run_dir,
                orchestrator: Some(server),
                _helper_nodes: Vec::new(),
            })
        }
        testing::Network::Mainnet => {
            let token = setup.lobby_token(stamp).to_string();
            let helpers = match setup {
                testing::harness::LobbySetup::Ensured => {
                    testing::harness::LobbyContractDeployer
                        .ensure_exists(&testing::harness::LobbyToken::from(token.as_str()))
                        .await
                        .map_err(|error| format!("lobby contract not ensured: {error:?}"))?
                }
                testing::harness::LobbySetup::Absent => Vec::new(),
            };
            Ok(testing::harness::BrpNetwork {
                network,
                token,
                deadlines,
                run_dir,
                orchestrator: None,
                _helper_nodes: helpers,
            })
        }
        testing::Network::LocalGateways => Err(
            "local-gateways is not implemented: NetworkArgs::validate rejects it, so no peer can start; use fake or mainnet"
                .to_string(),
        ),
        testing::Network::DirectDial => Ok(testing::harness::BrpNetwork {
            network,
            token: format!("brp-{}", core_harness::UnixNanos::now()),
            deadlines,
            run_dir,
            orchestrator: None,
            _helper_nodes: Vec::new(),
        }),
    }
}

// no test_usage necessary
