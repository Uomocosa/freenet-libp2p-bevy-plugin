use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use freenet_stdlib::client_api::ClientError;
use freenet_stdlib::client_api::ClientRequest;
use freenet_stdlib::client_api::ContractRequest;
use freenet_stdlib::client_api::ContractResponse;
use freenet_stdlib::client_api::HostResponse;
use freenet_stdlib::prelude::ContractContainer;
use freenet_stdlib::prelude::RelatedContracts;
use freenet_stdlib::prelude::WrappedState;
use freenet_stdlib::prelude::{ContractInstanceId, ContractKey};
use tokio_tungstenite::tungstenite::Message;

use crate::freenet;
use crate::identity;

pub struct Client {
    pub(crate) write: tokio::sync::mpsc::UnboundedSender<Message>,
    pub(crate) read: tokio::sync::mpsc::UnboundedReceiver<Result<HostResponse, ClientError>>,
}

impl Client {
    pub async fn from_host_and_port_with_retry(host: &str, port: u16) -> Self {
        let mut failures: u32 = 0;
        loop {
            match Self::try_from_host_and_port(host, port).await {
                Ok(client) => return client,
                Err(e) => {
                    failures = failures.saturating_add(1);
                    let random = RandomState::new().build_hasher().finish();
                    let delay = Self::reconnect_delay(failures, random);
                    tracing::warn!(
                        target: "freenet",
                        error = ?e,
                        failures,
                        delay_ms = delay.as_millis(),
                        "freenet websocket connect failed, retrying"
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

#[atomic_delegates]
impl Client {
    pub async fn try_from_host_and_port(host: &str, port: u16) -> Result<Self, freenet::Error> {}
    pub async fn fetch_or_deploy<S>(
        &mut self,
        contract_wasm: &[u8],
        params: &[u8],
        deploy: freenet::DeployPolicy,
        gateway_wait: Duration,
        seed_state: &S,
    ) -> Result<(ContractKey, S), freenet::Error>
    where
        S: serde::Serialize + serde::de::DeserializeOwned + Send + Sync,
    {
    }
    pub async fn wait_until_node_reachable(
        &mut self,
        budget: Duration,
    ) -> Result<(), freenet::Error> {
    }
    pub async fn connected_peers(&mut self) -> Result<Vec<identity::FreenetPeer>, freenet::Error> {}
    pub async fn node_status(&mut self) -> Result<freenet::NodeStatus, freenet::Error> {}
    pub async fn fetch_and_subscribe<S>(
        &mut self,
        instance_id: ContractInstanceId,
    ) -> Result<S, freenet::Error>
    where
        S: serde::de::DeserializeOwned,
    {
    }
    pub fn send(&self, request: &ClientRequest<'_>) -> Result<(), freenet::Error> {}
}

#[rustfmt::skip]
impl Client {
    pub async fn recv_reply(
        &mut self,
        kind: freenet::RequestKind,
    ) -> Result<HostResponse, freenet::Error> {
        let is_query = matches!(
            kind,
            freenet::RequestKind::QueryPeers
                | freenet::RequestKind::QueryNodeStatus
        );
        kind.run_with_timeout(async {
            loop {
                match self.next_reply().await? {
                    HostResponse::ContractResponse(
                        ContractResponse::SubscribeResponse { .. },
                    ) => {}
                    HostResponse::ContractResponse(
                        ContractResponse::GetResponse { .. } | ContractResponse::NotFound { .. },
                    ) if is_query => {}
                    other => return Ok(other),
                }
            }
        }).await
    }

    pub async fn await_outstanding_fetch<S>(&mut self) -> Result<S, freenet::Error>
    where
        S: serde::de::DeserializeOwned,
    {
        loop {
            match self.next_reply().await? {
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

    pub(crate) async fn next_reply(&mut self) -> Result<HostResponse, freenet::Error> {
        loop {
            match self.recv().await? {
                HostResponse::ContractResponse(ContractResponse::UpdateNotification { .. }) => {
                    tracing::debug!(target: "freenet", "contract update notification");
                }
                other => return Ok(other),
            }
        }
    }

    pub(crate) async fn deploy_contract<S>(
        &mut self,
        contract: ContractContainer,
        seed_state: &S,
    ) -> Result<usize, freenet::Error>
    where
        S: serde::Serialize + Send + Sync,
    {
        self.send(&ClientRequest::ContractOp(ContractRequest::Put {
            contract,
            state: WrappedState::new(bincode::serialize(seed_state)?),
            related_contracts: RelatedContracts::default(),
            subscribe: true,
            blocking_subscribe: false,
        }))?;
        let mut skipped_stale_get_replies: usize = 0;
        loop {
            match self.next_reply().await? {
                HostResponse::ContractResponse(
                    ContractResponse::PutResponse { key }
                    | ContractResponse::SubscribeResponse { key, .. }
                    | ContractResponse::UpdateResponse { key, .. },
                ) => {
                    tracing::info!(target: "freenet", key = %key, "freenet: contract deployed");
                    return Ok(skipped_stale_get_replies);
                }
                HostResponse::ContractResponse(
                    ContractResponse::GetResponse { .. } | ContractResponse::NotFound { .. },
                ) => {
                    skipped_stale_get_replies = skipped_stale_get_replies.saturating_add(1);
                }
                other => return Err(freenet::Error::UnexpectedResponse(format!("{other:?}"))),
            }
        }
    }

    pub(crate) fn send_lobby_fetch(
        &self,
        instance_id: ContractInstanceId,
    ) -> Result<(), freenet::Error> {
        let get = ContractRequest::Get {
            key: instance_id,
            return_contract_code: false,
            subscribe: true,
            blocking_subscribe: false,
        };
        self.send(&ClientRequest::ContractOp(get))
    }

    pub(crate) async fn fetch_after_deploy<S>(
        &mut self,
        instance_id: ContractInstanceId,
        stale_not_founds: usize,
    ) -> Result<S, freenet::Error>
    where
        S: serde::de::DeserializeOwned,
    {
        self.send_lobby_fetch(instance_id)?;
        let mut remaining_stale_not_founds = stale_not_founds;
        loop {
            match self.await_outstanding_fetch().await {
                Err(freenet::Error::ContractNotFound) if remaining_stale_not_founds > 0 => {
                    remaining_stale_not_founds = remaining_stale_not_founds.saturating_sub(1);
                }
                settled => return settled,
            }
        }
    }

    pub(crate) async fn recv(&mut self) -> Result<HostResponse, freenet::Error> {
        match self.read.recv().await {
            Some(Ok(response)) => Ok(response),
            Some(Err(e)) => Err(freenet::Error::from(e)),
            None => Err(freenet::Error::Disconnected),
        }
    }
}

#[atomic_delegates]
impl Client {
    pub(crate) fn reconnect_delay(failures: u32, random: u64) -> Duration {}
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use freenet_stdlib::client_api::{ContractResponse, HostResponse};
    use freenet_stdlib::prelude::{
        ContractCode, ContractContainer, ContractKey, ContractWasmAPIVersion, Parameters,
        WrappedContract, WrappedState,
    };
    use serde::{Deserialize, Serialize};

    use super::Client;
    use crate::freenet;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
    struct TestState {
        value: u64,
    }

    fn disconnected_client() -> Client {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        Client { write, read }
    }

    #[tokio::test]
    async fn test_usage() {
        let mut client = disconnected_client();
        assert!(client.recv().await.is_err());
        assert!(client.next_reply().await.is_err());
        let result = client.recv_reply(freenet::RequestKind::QueryPeers).await;
        assert!(matches!(result, Err(freenet::Error::Disconnected)));

        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let mut silent = Client { write, read };
        let result = silent.recv_reply(freenet::RequestKind::PollLobby).await;
        assert!(matches!(result, Err(freenet::Error::Timeout(_))));
    }

    fn contract_key() -> ContractKey {
        WrappedContract::new(
            Arc::new(ContractCode::from(vec![0; 4])),
            Parameters::from(Vec::new()),
        )
        .key
    }

    fn client_with(reply: HostResponse) -> Client {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx.send(Ok(reply)).ok();
        Client { write, read }
    }

    #[tokio::test]
    async fn outstanding_fetch_drains_state_or_absence() {
        let key = contract_key();
        let state = TestState { value: 7 };
        let mut found = client_with(HostResponse::ContractResponse(
            ContractResponse::GetResponse {
                key,
                contract: None,
                state: WrappedState::new(bincode::serialize(&state).unwrap_or_default()),
            },
        ));
        assert_eq!(
            found.await_outstanding_fetch::<TestState>().await.ok(),
            Some(state)
        );

        let mut missing = client_with(HostResponse::ContractResponse(ContractResponse::NotFound {
            instance_id: *key.id(),
        }));
        assert!(matches!(
            missing.await_outstanding_fetch::<TestState>().await,
            Err(freenet::Error::ContractNotFound)
        ));
    }

    #[tokio::test]
    async fn deploy_skips_stale_get_replies() {
        let key = contract_key();
        let seed = TestState { value: 1 };
        let wrapped = WrappedContract::new(
            Arc::new(ContractCode::from(vec![0; 4])),
            Parameters::from(Vec::new()),
        );
        let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::NotFound {
                    instance_id: *key.id(),
                },
            )))
            .ok();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::GetResponse {
                    key,
                    contract: None,
                    state: WrappedState::new(bincode::serialize(&seed).unwrap_or_default()),
                },
            )))
            .ok();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::PutResponse { key },
            )))
            .ok();
        let mut client = Client { write, read };
        assert_eq!(client.deploy_contract(container, &seed).await.ok(), Some(2));
        assert!(write_rx.recv().await.is_some());
    }

    #[tokio::test]
    async fn recv_reply_skips_stray_subscribe_response() {
        use freenet_stdlib::client_api::QueryResponse;
        let key = contract_key();
        let state = TestState { value: 3 };
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
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
                    state: WrappedState::new(bincode::serialize(&state).unwrap_or_default()),
                },
            )))
            .ok();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::NotFound {
                    instance_id: *key.id(),
                },
            )))
            .ok();
        read_tx
            .send(Ok(HostResponse::QueryResponse(
                QueryResponse::ConnectedPeers { peers: Vec::new() },
            )))
            .ok();
        let mut client = Client { write, read };
        let reply = client
            .recv_reply(freenet::RequestKind::QueryPeers)
            .await
            .ok();
        assert!(matches!(
            reply,
            Some(HostResponse::QueryResponse(
                QueryResponse::ConnectedPeers { .. }
            ))
        ));
    }

    #[tokio::test]
    async fn recv_reply_returns_get_for_poll() {
        let key = contract_key();
        let state = TestState { value: 5 };
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
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
                    state: WrappedState::new(bincode::serialize(&state).unwrap_or_default()),
                },
            )))
            .ok();
        let mut client = Client { write, read };
        let reply = client
            .recv_reply(freenet::RequestKind::PollLobby)
            .await
            .ok();
        assert!(matches!(
            reply,
            Some(HostResponse::ContractResponse(
                ContractResponse::GetResponse { .. }
            ))
        ));
        let _ = HashMap::<String, String>::new();
    }
}
