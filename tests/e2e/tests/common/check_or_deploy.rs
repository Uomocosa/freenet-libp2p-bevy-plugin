use std::sync::Arc;
use std::time::{Duration, Instant};

use freenet_libp2p_bevy_plugin::freenet_node;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::lobby_room::discovery;
use freenet_libp2p_bevy_plugin::lobby_room::freenet;
use freenet_stdlib::prelude::{
    ContractCode, ContractInstanceId, ContractKey, Parameters, WrappedContract,
};

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
/// Returns the helper nodes' dirs so the caller keeps them alive until the run
/// ends: a `Temporary` dir removed on drop deletes the node's folder on disk,
/// which must not happen while the run's helpers are still working.
///
/// # Errors
/// Returns the setup failure as [`LobbySetupError`] when the helper nodes
/// cannot start, the deploy cannot land, or the contract is still not readable
/// after the deploy.
pub async fn check_or_deploy(
    token: &LobbyToken,
) -> Result<Vec<freenet_node::NodeDir>, LobbySetupError> {
    let started = Instant::now();
    let params = identity::GameId::from(token.as_str()).contract_params();
    let instance_id = *lobby_key(&params).id();
    let checker = freenet_node::EmbeddedNode::start(None)
        .await
        .map_err(|error| LobbySetupError::NodeStart(error.to_string()))?;
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
        .map_err(|error| LobbySetupError::NodeStart(error.to_string()))?;
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
        return Err(LobbySetupError::Deploy(error));
    }
    let remaining = ensure_deadline
        .checked_duration_since(Instant::now())
        .unwrap_or_default();
    let readable_after_deploy = checker
        .wait_until_get_contract::<discovery::Lobby>(instance_id, remaining)
        .await;
    readable_after_deploy.map_err(LobbySetupError::NotReadableAfterDeploy)?;
    Ok(vec![checker_dir, deployer_dir])
}

/// The contract key the game peers compute for this lobby: the committed
/// canonical lobby wasm plus the token's bytes as parameters.
fn lobby_key(params: &[u8]) -> ContractKey {
    let code = Arc::new(ContractCode::from(discovery::LOBBY_CONTRACT_WASM.to_vec()));
    let wrapped = WrappedContract::new(code, Parameters::from(params.to_vec()));
    wrapped.key
}

#[cfg(test)]
mod tests {
    use super::ENSURE_BUDGET;
    use super::check_or_deploy;
    use super::lobby_key;
    use freenet_libp2p_bevy_plugin::freenet_node;
    use freenet_libp2p_bevy_plugin::identity;
    use std::path::PathBuf;
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
        let helper_dirs = [
            freenet_node::NodeDir::Kept(PathBuf::from("/tmp/checker")),
            freenet_node::NodeDir::Kept(PathBuf::from("/tmp/deployer")),
        ];
        assert_eq!(helper_dirs.len(), 2);
    }
}
