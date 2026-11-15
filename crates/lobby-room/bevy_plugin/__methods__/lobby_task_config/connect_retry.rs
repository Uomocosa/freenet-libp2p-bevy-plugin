use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

use tokio::sync::watch::Receiver;
use tracing::{debug, warn};

use crate::discovery;
use crate::freenet;

pub async fn connect_retry(
    state: &discovery::LobbyTaskConfig,
    target: &Receiver<Vec<discovery::state_machine::HeartbeatTarget>>,
) -> discovery::LobbyContractSubscription {
    let wasm = discovery::LOBBY_CONTRACT_WASM;
    let mut client = freenet::Client::from_host_and_port_with_retry(&state.host, state.port).await;
    let mut tries = 0_u32;
    loop {
        tries = tries.saturating_add(1);
        let current = target.borrow().clone();
        let seed = discovery::LobbyTaskConfig::seed_lobby_for_deploy(
            &state.me,
            state.discovery.room_size,
            current.first(),
        );
        let result = client
            .fetch_or_deploy(
                wasm,
                &state.params,
                state.discovery.deploy_policy,
                discovery::GATEWAY_WAIT,
                &seed,
            )
            .await;
        match result {
            Ok((key, lobby)) => {
                return discovery::LobbyContractSubscription {
                    client,
                    key,
                    lobby,
                    has_unread_subscribe_answer: false,
                };
            }
            Err(freenet::Error::Timeout(kind)) => {
                warn!(target: "discovery", tries = tries, kind = %kind, "{kind} timed out, reconnecting");
                client =
                    freenet::Client::from_host_and_port_with_retry(&state.host, state.port).await;
            }
            Err(freenet::Error::ContractNotFound) => {
                debug!(target: "discovery", tries = tries, "lobby absent and not in a room, waiting");
                tokio::time::sleep(discovery::MISSING_RETRY).await;
            }
            Err(e) => {
                warn!(target: "discovery", tries = tries, error = ?e, "lobby fetch or deploy failed, retrying");
                let random = RandomState::new().build_hasher().finish();
                let delay = freenet::Client::reconnect_delay(tries, random);
                tokio::time::sleep(delay).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::connect_retry;

    use crate::discovery;
    use crate::identity;

    #[tokio::test]
    async fn test_usage() {
        let config = discovery::LobbyTaskConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            me: identity::PeerId::from("me"),
            params: Vec::new(),
            discovery: discovery::Config::default(),
        };
        let (_target_tx, target_rx) = tokio::sync::watch::channel(Vec::new());
        let handle = tokio::spawn(async move { connect_retry(&config, &target_rx).await });
        handle.abort();
    }
}
