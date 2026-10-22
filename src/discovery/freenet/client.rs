use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use freenet_stdlib::client_api::ClientError;
use freenet_stdlib::client_api::ClientRequest;
use freenet_stdlib::client_api::ContractResponse;
use freenet_stdlib::client_api::HostResponse;
use freenet_stdlib::prelude::{ContractInstanceId, ContractKey};
use tokio_tungstenite::tungstenite::Message;

use crate::discovery;
use crate::identity;
use discovery::freenet::basic::constants;

pub struct Client {
    pub(crate) write: tokio::sync::mpsc::UnboundedSender<Message>,
    pub(crate) read: tokio::sync::mpsc::UnboundedReceiver<Result<HostResponse, ClientError>>,
}

#[atomic_delegates]
impl Client {
    pub async fn attach(
        &mut self,
        contract_wasm: &[u8],
        params: &[u8],
        deploy: discovery::freenet::DeployPolicy,
        ring_wait: Duration,
        initial_lobby: &discovery::Lobby,
    ) -> Result<(ContractKey, discovery::Lobby), discovery::Error> {
    }
    pub async fn wait_until_ring_ready(
        &mut self,
        budget: Duration,
    ) -> Result<(), discovery::Error> {
    }
    pub async fn connected_peers(
        &mut self,
    ) -> Result<Vec<identity::FreenetPeer>, discovery::Error> {
    }
    pub async fn fetch_and_subscribe(
        &mut self,
        instance_id: ContractInstanceId,
    ) -> Result<discovery::Lobby, discovery::Error> {
    }
    pub async fn try_from_host_and_port(host: &str, port: u16) -> Result<Self, discovery::Error> {}
    pub fn send(&self, request: &ClientRequest<'_>) -> Result<(), discovery::Error> {}
}

#[rustfmt::skip]
impl Client {
    pub async fn recv_reply(
        &mut self,
        kind: discovery::FreenetRequestKind,
        timeout: Duration,
    ) -> Result<HostResponse, discovery::Error> {
        discovery::freenet::RequestTimeout(timeout)
            .within(kind, self.next_reply())
            .await
    }

    pub(crate) async fn next_reply(&mut self) -> Result<HostResponse, discovery::Error> {
        loop {
            match self.recv().await? {
                HostResponse::ContractResponse(ContractResponse::UpdateNotification { .. }) => {}
                other => return Ok(other),
            }
        }
    }

    async fn recv(&mut self) -> Result<HostResponse, discovery::Error> {
        match self.read.recv().await {
            Some(Ok(response)) => Ok(response),
            Some(Err(e)) => Err(discovery::Error::from(e)),
            None => Err(discovery::Error::Disconnected),
        }
    }
}

impl Client {
    pub async fn from_host_and_port_with_retry(host: &str, port: u16) -> Self {
        loop {
            match Self::try_from_host_and_port(host, port).await {
                Ok(client) => return client,
                Err(e) => {
                    tracing::warn!(
                        target: "discovery",
                        error = %e,
                        "freenet websocket connect failed, retrying"
                    );
                    tokio::time::sleep(constants::CONNECT_RETRY).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Client;
    use crate::discovery;

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
        let result = client
            .recv_reply(
                discovery::FreenetRequestKind::QueryPeers,
                Duration::from_millis(1),
            )
            .await;
        assert!(matches!(result, Err(discovery::Error::Disconnected)));

        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let mut silent = Client { write, read };
        let result = silent
            .recv_reply(
                discovery::FreenetRequestKind::QueryPeers,
                Duration::from_millis(1),
            )
            .await;
        assert!(matches!(result, Err(discovery::Error::Timeout(_))));
    }
}
