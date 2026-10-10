use std::sync::Arc;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::discovery;

pub async fn connect(
    host: &str,
    port: u16,
    contract_wasm: &[u8],
    params: &[u8],
    deploy: discovery::freenet::DeployPolicy,
) -> Result<discovery::freenet::LobbyClient, discovery::Error> {
    let mut client = discovery::freenet::Client::connect(host, port).await?;
    let code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let wrapped = WrappedContract::new(code, Parameters::from(params.to_vec()));
    let key = wrapped.key;
    let instance_id = *key.id();
    let lobby = match discovery::freenet::fetch(&mut client, instance_id).await {
        Ok(lobby) => lobby,
        Err(discovery::Error::ContractNotFound)
            if deploy == discovery::freenet::DeployPolicy::FetchOrDeploy =>
        {
            let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
            deploy_contract(&mut client, container).await?;
            discovery::freenet::fetch(&mut client, instance_id).await?
        }
        Err(e) => return Err(e),
    };
    Ok(discovery::freenet::LobbyClient { client, key, lobby })
}

// needed helper: puts the lobby contract with an empty state and waits for the ack
async fn deploy_contract(
    client: &mut discovery::freenet::Client,
    contract: ContractContainer,
) -> Result<(), discovery::Error> {
    let put = ContractRequest::Put {
        contract,
        state: WrappedState::new(bincode::serialize(&discovery::Lobby::new())?),
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
    use super::connect;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        assert!(
            connect(
                "127.0.0.1",
                1,
                &[],
                &[],
                discovery::freenet::DeployPolicy::FetchOrDeploy,
            )
            .await
            .is_err()
        );
    }
}
