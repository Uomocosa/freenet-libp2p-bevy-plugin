use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use crate::discovery;
use crate::freenet;

pub async fn poll(
    state: &mut discovery::LobbyContractSubscription,
) -> Result<discovery::Lobby, discovery::Error> {
    loop {
        let reply = match freenet::RequestKind::PollLobby
            .run_with_timeout(state.client.next_reply())
            .await
        {
            Err(freenet::Error::Timeout(_)) => break,
            reply => reply?,
        };
        match reply {
            HostResponse::ContractResponse(ContractResponse::GetResponse {
                state: bytes, ..
            }) => {
                state.absorb_bytes(bytes.as_ref())?;
            }
            HostResponse::ContractResponse(ContractResponse::SubscribeResponse { .. }) => {
                state.has_unread_subscribe_answer = true;
            }
            _ => {}
        }
    }
    Ok(state.lobby.clone())
}
#[cfg(test)]
mod tests {
    use freenet_stdlib::client_api::{ContractResponse, HostResponse};
    use freenet_stdlib::prelude::WrappedState;

    use super::poll;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let key = testing::contract_key();
        let incoming = testing::room_lobby();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::GetResponse {
                    key,
                    contract: None,
                    state: WrappedState::new(bincode::serialize(&incoming).unwrap_or_default()),
                },
            )))
            .ok();
        let mut subscription = testing::lobby_subscription(write, read);
        let polled = poll(&mut subscription).await;
        assert_eq!(polled.ok(), Some(incoming));
        assert_eq!(subscription.lobby.len(), 1);
    }

    #[tokio::test]
    async fn poll_records_the_subscribe_answer() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let key = testing::contract_key();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::SubscribeResponse {
                    key,
                    subscribed: true,
                },
            )))
            .ok();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::GetResponse {
                    key,
                    contract: None,
                    state: WrappedState::new(
                        bincode::serialize(&testing::room_lobby()).unwrap_or_default(),
                    ),
                },
            )))
            .ok();
        let mut subscription = testing::lobby_subscription(write, read);
        poll(&mut subscription).await.ok();
        assert!(subscription.take_subscribe_answer());
        assert!(!subscription.take_subscribe_answer());
    }
}
