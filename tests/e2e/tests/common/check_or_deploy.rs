use std::sync::Arc;
use std::time::{Duration, Instant};

use freenet_libp2p_bevy_plugin::discovery;
use freenet_libp2p_bevy_plugin::freenet;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::testing::{Network, NetworkArgs};
use freenet_stdlib::prelude::{
    ContractCode, ContractInstanceId, ContractKey, Parameters, WrappedContract,
};

use examples_shared::node::start_embedded_freenet_node;

use super::LobbySetupError;
use super::LobbyToken;

/// Total budget one `check_or_deploy` may spend before the run gives up.
const ENSURE_BUDGET: Duration = Duration::from_secs(600);

/// How long the checker may poll for an existing contract before the run
/// concludes the contract is absent and deploys it from a second node.
const ABSENT_WINDOW: Duration = Duration::from_secs(150);

/// Pace between deploy retries when the first deploy attempt fails.
const DEPLOY_RETRY_AFTER: Duration = Duration::from_secs(5);

/// Room one deploy attempt needs (node reachable + fetch silence + a `Put`).
const DEPLOY_ATTEMPT_BUDGET: Duration = Duration::from_secs(120);

const HELPER_HOST: &str = "127.0.0.1";

/// Makes sure the lobby contract for `token` exists on the network before any
/// game peer spawns: a checker node that never deploys GETs the contract; when
/// the network reports it absent, a second, different embedded node deploys
/// `Lobby::default()`, and the checker GETs again — from its own copy only, so
/// "readable from the network" is proven by a node that never hosted it.
///
/// # Errors
/// Returns the setup failure as [`LobbySetupError`] when the helper nodes
/// cannot start, the deploy cannot land, or the contract is still not readable
/// after the deploy.
pub async fn check_or_deploy(token: &LobbyToken) -> Result<(), LobbySetupError> {
    let started = Instant::now();
    let params = identity::GameId::from(token.as_str()).contract_params();
    let instance_id = *lobby_key(&params).id();
    let (checker_dir, checker_port) = start_embedded_freenet_node(None, &helper_args())
        .await
        .map_err(LobbySetupError::NodeStart)?;
    let mut checker =
        freenet::Client::from_host_and_port_with_retry(HELPER_HOST, checker_port).await;
    let absent_budget = std::cmp::min(ENSURE_BUDGET, ABSENT_WINDOW);
    let readable = checker
        .wait_until_get_contract::<discovery::Lobby>(instance_id, absent_budget)
        .await;
    if readable.is_ok() {
        std::mem::forget(checker_dir);
        return Ok(());
    }
    let (deployer_dir, deployer_port) = start_embedded_freenet_node(None, &helper_args())
        .await
        .map_err(LobbySetupError::NodeStart)?;
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
        std::mem::forget(checker_dir);
        std::mem::forget(deployer_dir);
        return Err(LobbySetupError::Deploy(error));
    }
    std::mem::forget(checker_dir);
    std::mem::forget(deployer_dir);
    let remaining = ensure_deadline
        .checked_duration_since(Instant::now())
        .unwrap_or_default();
    let readable_after_deploy = checker
        .wait_until_get_contract::<discovery::Lobby>(instance_id, remaining)
        .await;
    readable_after_deploy.map_err(LobbySetupError::NotReadableAfterDeploy)?;
    Ok(())
}

/// The contract key the game peers compute for this lobby: the committed
/// canonical lobby wasm plus the token's bytes as parameters.
fn lobby_key(params: &[u8]) -> ContractKey {
    let code = Arc::new(ContractCode::from(discovery::LOBBY_CONTRACT_WASM.to_vec()));
    let wrapped = WrappedContract::new(code, Parameters::from(params.to_vec()));
    wrapped.key
}

/// The embedded helper nodes join the same network the game peers use.
const fn helper_args() -> NetworkArgs {
    NetworkArgs {
        network: Network::Mainnet,
        orchestrator_port: None,
    }
}

#[cfg(test)]
mod tests {
    use super::ENSURE_BUDGET;
    use super::check_or_deploy;
    use super::lobby_key;
    use freenet_libp2p_bevy_plugin::identity;
    use std::time::Instant;

    #[test]
    fn test_usage() {
        let token = identity::GameId::from("lobby-e2e/ensured");
        let key = lobby_key(&token.contract_params());
        assert_eq!(*key.id(), *lobby_key(&token.contract_params()).id());
        assert_ne!(
            *key.id(),
            *lobby_key(&identity::GameId::from("other").contract_params()).id()
        );
        assert_eq!(ENSURE_BUDGET.as_secs(), 600);
        let _ = check_or_deploy;
        let _ = Instant::now();
    }
}
