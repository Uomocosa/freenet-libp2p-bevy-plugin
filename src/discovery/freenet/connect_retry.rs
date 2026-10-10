use tracing::{debug, warn};

use crate::discovery;
use discovery::freenet::basic::constants;

pub async fn connect_retry(
    host: &str,
    port: u16,
    params: &[u8],
    target: &tokio::sync::watch::Receiver<Option<discovery::state_machine::PublishTarget>>,
) -> discovery::freenet::LobbyClient {
    let wasm = discovery::freenet::contract_wasm();
    let timeout = constants::REQUEST_TIMEOUT;
    loop {
        let deploy = if target.borrow().is_some() {
            discovery::freenet::DeployPolicy::FetchOrDeploy
        } else {
            discovery::freenet::DeployPolicy::FetchOnly
        };
        let attempt = discovery::freenet::connect(host, port, wasm, params, deploy);
        let Ok(result) = tokio::time::timeout(timeout, attempt).await else {
            warn!(target: "room_lobby", "discovery: lobby connect timed out, retrying");
            continue;
        };
        match result {
            Ok(lobby_client) => return lobby_client,
            Err(discovery::Error::ContractNotFound) => {
                debug!(target: "room_lobby", "discovery: no lobby yet and not in a room, waiting");
                tokio::time::sleep(constants::MISSING_RETRY).await;
            }
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: lobby connect failed, retrying");
                tokio::time::sleep(constants::CONNECT_RETRY).await;
            }
        }
    }
}

// no test_usage necessary
