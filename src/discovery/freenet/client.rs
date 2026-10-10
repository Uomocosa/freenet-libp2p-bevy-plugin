use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use freenet_stdlib::client_api::ClientError;
use freenet_stdlib::client_api::ClientRequest;
use freenet_stdlib::client_api::ContractResponse;
use freenet_stdlib::client_api::HostResponse;
use tokio_tungstenite::tungstenite::Message;

use crate::discovery;

pub struct Client {
    pub(crate) write: tokio::sync::mpsc::UnboundedSender<Message>,
    pub(crate) read: tokio::sync::mpsc::UnboundedReceiver<Result<HostResponse, ClientError>>,
}

#[atomic_delegates]
impl Client {
    pub async fn connect(host: &str, port: u16) -> Result<Self, discovery::Error> {}
    pub fn send(&self, request: &ClientRequest<'_>) -> Result<(), discovery::Error> {}
}

#[rustfmt::skip]
impl Client {
    pub async fn recv(&mut self) -> Result<HostResponse, discovery::Error> {
        match self.read.recv().await {
            Some(Ok(response)) => Ok(response),
            Some(Err(e)) => Err(discovery::Error::from(e)),
            None => Err(discovery::Error::Disconnected),
        }
    }

    pub async fn recv_response(&mut self) -> Result<HostResponse, discovery::Error> {
        loop {
            match self.recv().await? {
                HostResponse::ContractResponse(ContractResponse::UpdateNotification { .. }) => {}
                other => return Ok(other),
            }
        }
    }

    pub async fn recv_response_timeout(
        &mut self,
        timeout: Duration,
    ) -> Option<Result<HostResponse, discovery::Error>> {
        tokio::time::timeout(timeout, self.recv_response()).await.ok()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Client;

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
        assert!(client.recv_response().await.is_err());
        let mut client = disconnected_client();
        let result = client.recv_response_timeout(Duration::from_millis(1)).await;
        assert!(matches!(result, Some(Err(_))));
    }
}
