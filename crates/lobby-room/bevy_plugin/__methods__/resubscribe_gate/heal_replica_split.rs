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
    use std::time::Duration;

    use tokio::time::Instant;

    use super::heal_replica_split;
    use crate::discovery;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let (mut subscription, mut write_rx) = testing::live_lobby_subscription();
        let mut gate = discovery::ResubscribeGate::default();
        let started = Instant::now();
        heal_replica_split(&mut gate, &mut subscription, started);
        assert_eq!(testing::subscribes_sent(&mut write_rx), 1);
        heal_replica_split(&mut gate, &mut subscription, started);
        assert_eq!(testing::subscribes_sent(&mut write_rx), 0);
        heal_replica_split(
            &mut gate,
            &mut subscription,
            started + discovery::RESUBSCRIBE,
        );
        assert_eq!(testing::subscribes_sent(&mut write_rx), 1);
    }

    #[tokio::test]
    async fn a_send_error_is_not_counted_as_sent() {
        let (write, write_rx) = tokio::sync::mpsc::unbounded_channel();
        drop(write_rx);
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let mut subscription = testing::lobby_subscription(write, read);
        let mut gate = discovery::ResubscribeGate::default();
        let started = Instant::now();
        heal_replica_split(&mut gate, &mut subscription, started);
        assert!(gate.is_due(started + Duration::from_millis(1)));
    }

    #[tokio::test]
    async fn an_answer_resets_the_missed_sends() {
        let (mut subscription, mut write_rx) = testing::live_lobby_subscription();
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
        assert_eq!(testing::subscribes_sent(&mut write_rx), 0);
        heal_replica_split(
            &mut gate,
            &mut subscription,
            started + Duration::from_secs(30),
        );
        assert_eq!(testing::subscribes_sent(&mut write_rx), 1);
        assert!(!gate.is_due(
            started + Duration::from_secs(30) + discovery::RESUBSCRIBE - Duration::from_millis(1)
        ));
        assert!(gate.is_due(started + Duration::from_secs(30) + discovery::RESUBSCRIBE));
    }
}
