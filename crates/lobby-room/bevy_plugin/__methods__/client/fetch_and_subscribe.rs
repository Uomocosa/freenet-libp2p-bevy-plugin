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
    use crate::freenet;
    use crate::testing;

    use super::fetch_and_subscribe;

    #[tokio::test]
    async fn test_usage() {
        let key = testing::contract_key();
        let state = testing::TestState(7);
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx
            .send(Ok(testing::contract_get_response(key, &state)))
            .ok();
        let mut client = freenet::Client { write, read };
        assert_eq!(
            fetch_and_subscribe::<testing::TestState>(&mut client, *key.id())
                .await
                .ok(),
            Some(state)
        );

        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx
            .send(Ok(testing::contract_not_found_response(key)))
            .ok();
        let mut missing = freenet::Client { write, read };
        assert!(matches!(
            fetch_and_subscribe::<testing::TestState>(&mut missing, *key.id()).await,
            Err(freenet::Error::ContractNotFound)
        ));
    }
}
