use tokio::sync::mpsc::UnboundedReceiver;
use tokio_tungstenite::tungstenite::Message;

use crate::discovery;
use crate::testing;

#[must_use]
pub fn live_lobby_subscription() -> (
    discovery::LobbyContractSubscription,
    UnboundedReceiver<Message>,
) {
    let (write, written) = tokio::sync::mpsc::unbounded_channel();
    let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
    drop(read_tx);
    (testing::lobby_subscription(write, read), written)
}

#[cfg(test)]
mod tests {
    use crate::testing;

    use super::live_lobby_subscription;

    #[test]
    fn test_usage() {
        let (subscription, mut written) = live_lobby_subscription();
        assert_eq!(subscription.key, testing::contract_key());
        assert!(written.try_recv().is_err());
    }
}
