use std::sync::Arc;
use std::time::Duration;

use freenet_stdlib::prelude::*;
use tracing::{info, warn};

use crate::discovery;

pub async fn fetch_or_deploy_lobby(
    state: &mut discovery::freenet::Client,
    contract_wasm: &[u8],
    params: &[u8],
    deploy: discovery::freenet::DeployPolicy,
    ring_wait: Duration,
    seed_lobby: &discovery::Lobby,
) -> Result<(ContractKey, discovery::Lobby), discovery::Error> {
    state.wait_until_ring_ready(ring_wait).await?;
    let code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let wrapped = WrappedContract::new(code, Parameters::from(params.to_vec()));
    let key = wrapped.key;
    let instance_id = *key.id();
    info!(target: "discovery", %key, ?deploy, "lobby fetch start");
    let started = std::time::Instant::now();
    let first = discovery::freenet::RequestKind::FetchLobby
        .run_with_timeout(state.fetch_and_subscribe(instance_id))
        .await;
    let elapsed_ms = started.elapsed().as_millis();
    match &first {
        Ok(lobby) => {
            info!(target: "discovery", elapsed_ms, rooms = lobby.len(), "lobby fetch found");
        }
        Err(e) => {
            warn!(target: "discovery", elapsed_ms, error = ?e, "lobby fetch failed");
        }
    }
    let lobby = match first {
        Ok(lobby) => lobby,
        Err(discovery::Error::ContractNotFound)
            if deploy == discovery::freenet::DeployPolicy::FetchOrDeploy =>
        {
            info!(target: "discovery", %key, rooms = seed_lobby.len(), "lobby deploy start");
            let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
            discovery::freenet::RequestKind::DeployLobby
                .run_with_timeout(state.deploy_contract(container, seed_lobby))
                .await
                .inspect_err(|e| warn!(target: "discovery", error = ?e, "lobby deploy failed"))?;
            discovery::freenet::RequestKind::FetchLobby
                .run_with_timeout(state.fetch_and_subscribe(instance_id))
                .await?
        }
        Err(e) => return Err(e),
    };
    let node = state.node_status().await.unwrap_or_default();
    info!(
        target: "discovery",
        %key,
        rooms = lobby.len(),
        ring_peers = node.ring_peers,
        transport_peers = node.transport_peers,
        ring_location = ?node.ring_location,
        "lobby ready"
    );
    Ok((key, lobby))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::fetch_or_deploy_lobby;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = discovery::freenet::Client { write, read };
        let lobby = discovery::Lobby::default();
        assert!(
            fetch_or_deploy_lobby(
                &mut client,
                &[],
                &[],
                discovery::freenet::DeployPolicy::FetchOrDeploy,
                Duration::from_millis(1),
                &lobby,
            )
            .await
            .is_err()
        );
    }
}
