use std::time::Duration;

use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use crate::discovery;

const DRAIN_TIMEOUT: Duration = Duration::from_millis(10);

pub async fn poll(
    subscription: &mut discovery::freenet::LobbyContractSubscription,
) -> Result<discovery::Lobby, discovery::Error> {
    while let Some(result) = subscription
        .client
        .recv_response_timeout(DRAIN_TIMEOUT)
        .await
    {
        if let HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) = result?
        {
            absorb_bytes(subscription, state.as_ref())?;
        }
    }
    Ok(subscription.lobby.clone())
}

// needed helper: merges raw lobby bytes into the cached lobby
fn absorb_bytes(
    subscription: &mut discovery::freenet::LobbyContractSubscription,
    bytes: &[u8],
) -> Result<(), discovery::Error> {
    let incoming: discovery::Lobby = bincode::deserialize(bytes)?;
    subscription.lobby.merge(incoming);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use freenet_stdlib::client_api::{ContractResponse, HostResponse};
    use freenet_stdlib::prelude::{
        ContractCode, ContractKey, Parameters, WrappedContract, WrappedState,
    };

    use super::poll;
    use crate::{discovery, identity};

    fn contract_key() -> ContractKey {
        WrappedContract::new(
            Arc::new(ContractCode::from(
                discovery::freenet::contract_wasm().to_vec(),
            )),
            Parameters::from(Vec::new()),
        )
        .key
    }

    fn room_lobby() -> discovery::Lobby {
        discovery::Lobby::from([(
            identity::RoomName::from("room-a"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                members: BTreeMap::new(),
            },
        )])
    }

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let key = contract_key();
        let incoming = room_lobby();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::GetResponse {
                    key,
                    contract: None,
                    state: WrappedState::new(bincode::serialize(&incoming).unwrap_or_default()),
                },
            )))
            .ok();
        let mut subscription = discovery::freenet::LobbyContractSubscription {
            client: discovery::freenet::Client { write, read },
            key,
            lobby: discovery::Lobby::default(),
        };
        let polled = poll(&mut subscription).await;
        assert_eq!(polled.ok(), Some(incoming));
        assert_eq!(subscription.lobby.len(), 1);
    }
}
