use std::time::Duration;

use freenet_stdlib::client_api::{ContractResponse, HostResponse};
use freenet_stdlib::prelude::UpdateData;

use crate::discovery;

const DRAIN_TIMEOUT: Duration = Duration::from_millis(10);

pub async fn poll(
    lobby_contract: &mut discovery::freenet::LobbyContract,
) -> Result<discovery::Lobby, discovery::Error> {
    while let Some(result) = lobby_contract
        .client
        .recv_response_timeout(DRAIN_TIMEOUT)
        .await
    {
        match result? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification {
                update, ..
            }) => {
                absorb_update(lobby_contract, update)?;
            }
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                absorb_bytes(lobby_contract, state.as_ref())?;
            }
            _ => {}
        }
    }
    Ok(lobby_contract.lobby.clone())
}

// needed helper: merges one notification into the cached lobby
fn absorb_update(
    lobby_contract: &mut discovery::freenet::LobbyContract,
    update: UpdateData<'static>,
) -> Result<(), discovery::Error> {
    let bytes = match update {
        UpdateData::State(state) | UpdateData::StateAndDelta { state, .. } => {
            Some(state.as_ref().to_vec())
        }
        UpdateData::Delta(delta) => Some(delta.as_ref().to_vec()),
        _ => None,
    };
    if let Some(bytes) = bytes {
        absorb_bytes(lobby_contract, &bytes)?;
    }
    Ok(())
}

// needed helper: merges raw lobby bytes into the cached lobby
fn absorb_bytes(
    lobby_contract: &mut discovery::freenet::LobbyContract,
    bytes: &[u8],
) -> Result<(), discovery::Error> {
    let incoming: discovery::Lobby = bincode::deserialize(bytes)?;
    discovery::freenet::merge_cached_lobby(lobby_contract, incoming);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        std::hint::black_box(super::poll);
    }
}
