use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use crate::freenet;

pub async fn await_outstanding_fetch<S>(client: &mut freenet::Client) -> Result<S, freenet::Error>
where
    S: serde::de::DeserializeOwned,
{
    loop {
        match client.next_reply().await? {
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                return Ok(bincode::deserialize(state.as_ref())?);
            }
            HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => {
                return Err(freenet::Error::ContractNotFound);
            }
            HostResponse::ContractResponse(ContractResponse::SubscribeResponse { .. }) => {}
            other => return Err(freenet::Error::UnexpectedResponse(format!("{other:?}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use freenet_stdlib::client_api::{ContractResponse, HostResponse};
    use freenet_stdlib::prelude::{ContractCode, Parameters, WrappedContract, WrappedState};

    use super::await_outstanding_fetch;
    use crate::freenet;

    #[tokio::test]
    async fn test_usage() {
        let key = WrappedContract::new(
            Arc::new(ContractCode::from(vec![0; 4])),
            Parameters::from(Vec::new()),
        )
        .key;
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::GetResponse {
                    key,
                    contract: None,
                    state: WrappedState::new(bincode::serialize(&7u64).unwrap_or_default()),
                },
            )))
            .ok();
        let mut client = freenet::Client { write, read };
        assert_eq!(
            await_outstanding_fetch::<u64>(&mut client).await.ok(),
            Some(7)
        );
    }
}
