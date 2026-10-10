use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::ContractInstanceId;

use crate::discovery;

pub async fn fetch(
    client: &mut discovery::freenet::Client,
    instance_id: ContractInstanceId,
) -> Result<discovery::Lobby, discovery::Error> {
    let get = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(&ClientRequest::ContractOp(get))?;
    loop {
        match client.recv_response().await? {
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

// no test_usage necessary
