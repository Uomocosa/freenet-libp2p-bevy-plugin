use std::sync::Arc;
use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::discovery;
use discovery::freenet::basic::constants::{DEPLOY_TIMEOUT, REQUEST_TIMEOUT};

pub async fn attach(
    client: &mut discovery::freenet::Client,
    contract_wasm: &[u8],
    params: &[u8],
    deploy: discovery::freenet::DeployPolicy,
    ring_wait: Duration,
    initial_board: &discovery::Lobby,
) -> Result<(ContractKey, discovery::Lobby), discovery::Error> {
    discovery::freenet::wait_until_ring_ready(client, ring_wait).await?;
    let code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let wrapped = WrappedContract::new(code, Parameters::from(params.to_vec()));
    let key = wrapped.key;
    let instance_id = *key.id();
    let first = discovery::within(
        REQUEST_TIMEOUT,
        "fetch",
        discovery::freenet::fetch(client, instance_id),
    )
    .await;
    let lobby = match first {
        Ok(lobby) => lobby,
        Err(discovery::Error::ContractNotFound)
            if deploy == discovery::freenet::DeployPolicy::FetchOrDeploy =>
        {
            let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
            discovery::within(
                DEPLOY_TIMEOUT,
                "deploy",
                deploy_contract(client, container, initial_board),
            )
            .await?;
            discovery::within(
                REQUEST_TIMEOUT,
                "fetch",
                discovery::freenet::fetch(client, instance_id),
            )
            .await?
        }
        Err(e) => return Err(e),
    };
    let peers = discovery::freenet::connected_peer_count(client)
        .await
        .unwrap_or_default();
    info!(
        target: "room_lobby",
        %key,
        rooms = lobby.len(),
        "lobby freenet ring ready peers={peers}"
    );
    Ok((key, lobby))
}

// needed helper: puts the lobby contract seeded with our own presence and waits for the ack
async fn deploy_contract(
    client: &mut discovery::freenet::Client,
    contract: ContractContainer,
    initial_board: &discovery::Lobby,
) -> Result<(), discovery::Error> {
    let put = ContractRequest::Put {
        contract,
        state: WrappedState::new(bincode::serialize(initial_board)?),
        related_contracts: RelatedContracts::default(),
        subscribe: true,
        blocking_subscribe: false,
    };
    client.send(&ClientRequest::ContractOp(put))?;
    match client.recv_response().await? {
        HostResponse::ContractResponse(
            ContractResponse::PutResponse { key }
            | ContractResponse::SubscribeResponse { key, .. }
            | ContractResponse::UpdateResponse { key, .. },
        ) => {
            info!(target: "room_lobby", key = %key, "discovery: lobby contract deployed");
            Ok(())
        }
        other => Err(discovery::Error::UnexpectedResponse(format!("{other:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::attach;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut client = discovery::freenet::Client { write, read };
        let board = discovery::Lobby::new();
        assert!(
            attach(
                &mut client,
                &[],
                &[],
                discovery::freenet::DeployPolicy::FetchOrDeploy,
                Duration::from_millis(1),
                &board,
            )
            .await
            .is_err()
        );
    }
}
