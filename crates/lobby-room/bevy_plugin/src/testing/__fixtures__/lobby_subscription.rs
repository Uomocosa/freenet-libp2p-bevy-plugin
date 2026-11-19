use freenet_stdlib::client_api::ClientError;
use freenet_stdlib::client_api::HostResponse;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

use crate::discovery;
use crate::freenet;
use crate::testing;

#[must_use]
pub fn lobby_subscription(
    write: UnboundedSender<Message>,
    read: UnboundedReceiver<Result<HostResponse, ClientError>>,
) -> discovery::LobbyContractSubscription {
    discovery::LobbyContractSubscription {
        client: freenet::Client { write, read },
        key: testing::contract_key(),
        lobby: discovery::Lobby::default(),
        has_unread_subscribe_answer: false,
    }
}

#[cfg(test)]
mod tests {
    use crate::testing;

    use super::lobby_subscription;

    #[test]
    fn test_usage() {
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let subscription = lobby_subscription(write, read);
        assert_eq!(subscription.key, testing::contract_key());
        assert!(subscription.lobby.is_empty());
        assert!(!subscription.has_unread_subscribe_answer);
    }
}
