use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::ContractInstanceId;

use crate::discovery;

pub async fn fetch_and_subscribe(
    state: &mut discovery::freenet::Client,
    instance_id: ContractInstanceId,
) -> Result<discovery::Lobby, discovery::Error> {
    let get = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    state.send(&ClientRequest::ContractOp(get))?;
    loop {
        match state.recv_response().await? {
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                return Ok(bincode::deserialize(state.as_ref())?);
            }
            HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => {
                return Err(discovery::Error::ContractNotFound);
            }
            HostResponse::ContractResponse(ContractResponse::SubscribeResponse { .. }) => {}
            other => return Err(discovery::Error::UnexpectedResponse(format!("{other:?}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use freenet_stdlib::client_api::{ContractResponse, HostResponse};
    use freenet_stdlib::prelude::{
        ContractCode, ContractKey, Parameters, WrappedContract, WrappedState,
    };

    use super::fetch_and_subscribe;
    use crate::discovery;

    fn contract_key() -> ContractKey {
        WrappedContract::new(
            Arc::new(ContractCode::from(
                discovery::freenet::LOBBY_CONTRACT_WASM.to_vec(),
            )),
            Parameters::from(Vec::new()),
        )
        .key
    }

    #[tokio::test]
    async fn test_usage() {
        let key = contract_key();
        let lobby = discovery::Lobby::default();
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::GetResponse {
                    key,
                    contract: None,
                    state: WrappedState::new(bincode::serialize(&lobby).unwrap_or_default()),
                },
            )))
            .ok();
        let mut client = discovery::freenet::Client { write, read };
        assert_eq!(
            fetch_and_subscribe(&mut client, *key.id()).await.ok(),
            Some(lobby)
        );

        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::NotFound {
                    instance_id: *key.id(),
                },
            )))
            .ok();
        let mut missing = discovery::freenet::Client { write, read };
        assert!(matches!(
            fetch_and_subscribe(&mut missing, *key.id()).await,
            Err(discovery::Error::ContractNotFound)
        ));
    }
}
