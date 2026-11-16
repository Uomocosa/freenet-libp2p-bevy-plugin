use std::time::{Duration, Instant};

use freenet_libp2p_bevy_core::freenet_node;
use freenet_libp2p_bevy_core::identity;

use crate::discovery;
use crate::freenet;
use crate::testing;

const ENSURE_BUDGET: Duration = Duration::from_secs(600);

const ABSENT_WINDOW: Duration = Duration::from_secs(150);

const DEPLOY_RETRY_AFTER: Duration = Duration::from_secs(5);

const DEPLOY_ATTEMPT_BUDGET: Duration = Duration::from_secs(120);

const HELPER_HOST: &str = "127.0.0.1";

pub async fn ensure_exists(
    _deployer: &testing::harness::LobbyContractDeployer,
    token: &testing::harness::LobbyToken,
) -> Result<Vec<freenet_node::NodeDir>, testing::harness::LobbySetupError> {
    let started = Instant::now();
    let params = identity::GameId::from(token.as_str()).contract_params();
    let instance_id = *testing::harness::LobbyContractDeployer::lobby_key(&params).id();
    let checker = freenet_node::EmbeddedNode::start(None)
        .await
        .map_err(testing::harness::LobbySetupError::NodeStart)?;
    let (checker_dir, checker_port) = (checker.node_dir, checker.websocket_port);
    let mut checker =
        freenet::Client::from_host_and_port_with_retry(HELPER_HOST, checker_port).await;
    let absent_budget = std::cmp::min(ENSURE_BUDGET, ABSENT_WINDOW);
    let readable = checker
        .wait_until_get_contract::<discovery::Lobby>(instance_id, absent_budget)
        .await;
    if readable.is_ok() {
        return Ok(vec![checker_dir]);
    }
    let deployer = freenet_node::EmbeddedNode::start(None)
        .await
        .map_err(testing::harness::LobbySetupError::NodeStart)?;
    let (deployer_dir, deployer_port) = (deployer.node_dir, deployer.websocket_port);
    let mut deployer =
        freenet::Client::from_host_and_port_with_retry(HELPER_HOST, deployer_port).await;
    let mut deploy_error = None;
    let ensure_deadline = started
        .checked_add(ENSURE_BUDGET)
        .unwrap_or_else(Instant::now);
    while Instant::now()
        .checked_add(DEPLOY_ATTEMPT_BUDGET)
        .is_some_and(|attempt_floor| attempt_floor < ensure_deadline)
    {
        let attempt = deployer
            .deploy(
                discovery::LOBBY_CONTRACT_WASM,
                &params,
                &discovery::Lobby::default(),
                discovery::GATEWAY_WAIT,
            )
            .await;
        match attempt {
            Ok(_) => {
                deploy_error = None;
                break;
            }
            Err(error) => {
                deploy_error = Some(error);
                deployer =
                    freenet::Client::from_host_and_port_with_retry(HELPER_HOST, deployer_port)
                        .await;
                tokio::time::sleep(DEPLOY_RETRY_AFTER).await;
            }
        }
    }
    if let Some(error) = deploy_error {
        return Err(testing::harness::LobbySetupError::Deploy(error));
    }
    let remaining = ensure_deadline
        .checked_duration_since(Instant::now())
        .unwrap_or_default();
    let readable_after_deploy = checker
        .wait_until_get_contract::<discovery::Lobby>(instance_id, remaining)
        .await;
    readable_after_deploy.map_err(testing::harness::LobbySetupError::NotReadableAfterDeploy)?;
    Ok(vec![checker_dir, deployer_dir])
}

// no test_usage necessary
