use crate::discovery;

pub async fn refresh(
    state: &mut discovery::LobbyContractSubscription,
) -> Result<discovery::Lobby, discovery::Error> {
    let instance_id = *state.key.id();
    let fresh = state
        .client
        .fetch_and_subscribe::<discovery::Lobby>(instance_id)
        .await?;
    state.lobby.merge(fresh);
    Ok(state.lobby.clone())
}
#[cfg(test)]
mod tests {
    use freenet_stdlib::client_api::{ContractResponse, HostResponse};
    use freenet_stdlib::prelude::WrappedState;

    use super::refresh;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let key = testing::contract_key();
        let fresh = testing::room_lobby();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::GetResponse {
                    key,
                    contract: None,
                    state: WrappedState::new(bincode::serialize(&fresh).unwrap_or_default()),
                },
            )))
            .ok();
        let mut subscription = testing::lobby_subscription(write, read);
        assert_eq!(refresh(&mut subscription).await.ok(), Some(fresh.clone()));
        assert_eq!(subscription.lobby, fresh);
    }
}
