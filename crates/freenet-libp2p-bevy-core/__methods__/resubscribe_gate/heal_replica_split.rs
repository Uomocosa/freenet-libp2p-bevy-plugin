use tokio::time::Instant;

use crate::discovery;

pub fn heal_replica_split(
    gate: &mut discovery::ResubscribeGate,
    subscription: &mut discovery::LobbyContractSubscription,
    now: Instant,
) {
    if subscription.take_subscribe_answer() {
        let missed_sends = gate.answered();
        if missed_sends > 0 {
            tracing::info!(
                target: "discovery",
                missed_sends,
                "lobby resubscribe answered after misses"
            );
        }
    }
    if gate.is_due(now) {
        match subscription.resubscribe_to_heal_replica_split() {
            Ok(()) => gate.sent(now),
            Err(error) => {
                tracing::warn!(target: "discovery", ?error, "lobby resubscribe failed");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use freenet_stdlib::client_api::{ClientError, ClientRequest, ContractRequest, HostResponse};
    use freenet_stdlib::prelude::{ContractCode, Parameters, WrappedContract};
    use tokio::sync::mpsc::UnboundedReceiver;
    use tokio::time::Instant;
    use tokio_tungstenite::tungstenite::Message;

    use super::heal_replica_split;
    use crate::discovery;
    use crate::freenet;

    fn subscription_with(
        write: tokio::sync::mpsc::UnboundedSender<Message>,
        read: UnboundedReceiver<Result<HostResponse, ClientError>>,
    ) -> discovery::LobbyContractSubscription {
        discovery::LobbyContractSubscription {
            client: freenet::Client { write, read },
            key: WrappedContract::new(
                Arc::new(ContractCode::from(vec![0; 4])),
                Parameters::from(Vec::new()),
            )
            .key,
            lobby: discovery::Lobby::default(),
            has_unread_subscribe_answer: false,
        }
    }

    fn live_subscription() -> (
        discovery::LobbyContractSubscription,
        UnboundedReceiver<Message>,
    ) {
        let (write, write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        (subscription_with(write, read), write_rx)
    }

    fn count_subscribes(write_rx: &mut UnboundedReceiver<Message>) -> u32 {
        let mut subscribes: u32 = 0;
        while let Ok(Message::Binary(bytes)) = write_rx.try_recv() {
            if let Ok(ClientRequest::ContractOp(ContractRequest::Subscribe { .. })) =
                bincode::deserialize::<ClientRequest<'_>>(&bytes)
            {
                subscribes = subscribes.saturating_add(1);
            }
        }
        subscribes
    }

    #[tokio::test]
    async fn test_usage() {
        let (mut subscription, mut write_rx) = live_subscription();
        let mut gate = discovery::ResubscribeGate::default();
        let started = Instant::now();
        heal_replica_split(&mut gate, &mut subscription, started);
        assert_eq!(count_subscribes(&mut write_rx), 1);
        heal_replica_split(&mut gate, &mut subscription, started);
        assert_eq!(count_subscribes(&mut write_rx), 0);
        heal_replica_split(
            &mut gate,
            &mut subscription,
            started + discovery::RESUBSCRIBE,
        );
        assert_eq!(count_subscribes(&mut write_rx), 1);
    }

    #[tokio::test]
    async fn a_send_error_is_not_counted_as_sent() {
        let (write, write_rx) = tokio::sync::mpsc::unbounded_channel();
        drop(write_rx);
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut subscription = subscription_with(write, read);
        let mut gate = discovery::ResubscribeGate::default();
        let started = Instant::now();
        heal_replica_split(&mut gate, &mut subscription, started);
        assert!(gate.is_due(started + Duration::from_millis(1)));
    }

    #[tokio::test]
    async fn an_answer_resets_the_missed_sends() {
        let (mut subscription, mut write_rx) = live_subscription();
        let mut gate = discovery::ResubscribeGate::default();
        let started = Instant::now();
        gate.sent(started);
        gate.sent(started);
        subscription.has_unread_subscribe_answer = true;
        heal_replica_split(
            &mut gate,
            &mut subscription,
            started + Duration::from_secs(5),
        );
        assert!(!subscription.take_subscribe_answer());
        assert_eq!(count_subscribes(&mut write_rx), 0);
        heal_replica_split(
            &mut gate,
            &mut subscription,
            started + Duration::from_secs(30),
        );
        assert_eq!(count_subscribes(&mut write_rx), 1);
        assert!(!gate.is_due(
            started + Duration::from_secs(30) + discovery::RESUBSCRIBE - Duration::from_millis(1)
        ));
        assert!(gate.is_due(started + Duration::from_secs(30) + discovery::RESUBSCRIBE));
    }
}
