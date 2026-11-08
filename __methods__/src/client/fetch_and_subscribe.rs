use freenet_stdlib::prelude::ContractInstanceId;

use crate::freenet;

pub async fn fetch_and_subscribe<S>(
    state: &mut freenet::Client,
    instance_id: ContractInstanceId,
) -> Result<S, freenet::Error>
where
    S: serde::de::DeserializeOwned,
{
    state.send_lobby_fetch(instance_id)?;
    state.await_outstanding_fetch().await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use freenet_stdlib::client_api::{ContractResponse, HostResponse};
    use freenet_stdlib::prelude::{
        ContractCode, ContractKey, Parameters, WrappedContract, WrappedState,
    };
    use serde::{Deserialize, Serialize};

    use super::fetch_and_subscribe;
    use crate::freenet;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
    struct TestState {
        value: u64,
    }

    fn contract_key() -> ContractKey {
        WrappedContract::new(
            Arc::new(ContractCode::from(vec![0; 4])),
            Parameters::from(Vec::new()),
        )
        .key
    }

    #[tokio::test]
    async fn test_usage() {
        let key = contract_key();
        let state = TestState { value: 7 };
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::GetResponse {
                    key,
                    contract: None,
                    state: WrappedState::new(bincode::serialize(&state).unwrap_or_default()),
                },
            )))
            .ok();
        let mut client = freenet::Client { write, read };
        assert_eq!(
            fetch_and_subscribe::<TestState>(&mut client, *key.id())
                .await
                .ok(),
            Some(state)
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
        let mut missing = freenet::Client { write, read };
        assert!(matches!(
            fetch_and_subscribe::<TestState>(&mut missing, *key.id()).await,
            Err(freenet::Error::ContractNotFound)
        ));
    }
}
