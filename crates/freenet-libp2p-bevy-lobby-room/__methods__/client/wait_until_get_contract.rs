use std::time::Duration;

use freenet_stdlib::prelude::ContractInstanceId;
use tracing::{info, warn};

use crate::freenet;
use freenet::Client;
use freenet::Error;
use freenet::GET_RETRY_AFTER;
use freenet::RequestKind;

pub async fn wait_until_get_contract<S>(
    state: &mut Client,
    instance_id: ContractInstanceId,
    budget: Duration,
) -> Result<S, Error>
where
    S: serde::de::DeserializeOwned,
{
    let deadline = tokio::time::Instant::now()
        .checked_add(budget)
        .unwrap_or_else(tokio::time::Instant::now);
    let started = tokio::time::Instant::now();
    let mut attempts: u32 = 0;
    loop {
        attempts = attempts.saturating_add(1);
        let sent = state.send_lobby_fetch(instance_id);
        match sent {
            Ok(()) => {}
            Err(error) => {
                warn!(
                    target: "freenet",
                    attempts,
                    elapsed_ms = started.elapsed().as_millis(),
                    error = ?error,
                    "contract fetch send failed"
                );
                if tokio::time::Instant::now() >= deadline {
                    return Err(error);
                }
                tokio::time::sleep(GET_RETRY_AFTER).await;
                continue;
            }
        }
        let settled = RequestKind::FetchLobby
            .run_with_timeout(state.await_outstanding_fetch::<S>())
            .await;
        match settled {
            Ok(found) => {
                info!(
                    target: "freenet",
                    attempts,
                    elapsed_ms = started.elapsed().as_millis(),
                    "contract fetch found"
                );
                return Ok(found);
            }
            Err(error) => {
                warn!(
                    target: "freenet",
                    attempts,
                    elapsed_ms = started.elapsed().as_millis(),
                    error = ?error,
                    "contract fetch attempt failed"
                );
                if tokio::time::Instant::now() >= deadline {
                    return Err(error);
                }
                tokio::time::sleep(GET_RETRY_AFTER).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use freenet_stdlib::client_api::{ContractResponse, HostResponse};
    use freenet_stdlib::prelude::{
        ContractCode, ContractKey, Parameters, WrappedContract, WrappedState,
    };
    use serde::{Deserialize, Serialize};

    use super::wait_until_get_contract;
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
        let state = TestState { value: 9 };
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
            wait_until_get_contract::<TestState>(&mut client, *key.id(), Duration::from_secs(60))
                .await
                .ok(),
            Some(state)
        );

        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut disconnected = freenet::Client { write, read };
        assert!(
            wait_until_get_contract::<TestState>(&mut disconnected, *key.id(), Duration::ZERO)
                .await
                .is_err()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn absent_by_deadline_fails_with_not_found() {
        let key = contract_key();
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        for _ in 0..32 {
            read_tx
                .send(Ok(HostResponse::ContractResponse(
                    ContractResponse::NotFound {
                        instance_id: *key.id(),
                    },
                )))
                .ok();
        }
        let mut client = freenet::Client { write, read };
        let result =
            wait_until_get_contract::<TestState>(&mut client, *key.id(), Duration::from_secs(10))
                .await;
        assert!(matches!(result, Err(freenet::Error::ContractNotFound)));
    }

    #[tokio::test(start_paused = true)]
    async fn timed_out_attempt_is_retried_until_found() {
        let key = contract_key();
        let state = TestState { value: 4 };
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let mut client = freenet::Client { write, read };
        let wait = async {
            wait_until_get_contract::<TestState>(&mut client, *key.id(), Duration::from_secs(60))
                .await
        };
        let driver = async {
            write_rx.recv().await.expect("first get sent");
            tokio::time::advance(freenet::FETCH_TIMEOUT).await;
            read_tx
                .send(Ok(HostResponse::ContractResponse(
                    ContractResponse::GetResponse {
                        key,
                        contract: None,
                        state: WrappedState::new(bincode::serialize(&state).unwrap_or_default()),
                    },
                )))
                .ok();
        };
        let (result, ()) = tokio::join!(wait, driver);
        assert_eq!(result.ok(), Some(state));
    }
}
