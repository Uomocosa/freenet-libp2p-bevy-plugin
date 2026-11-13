use tokio::time::Instant;

use crate::discovery;

use atomic_delegate_macros::atomic_delegates;

#[derive(Default)]
pub struct ResubscribeGate {
    pub(crate) next_due: Option<Instant>,
    pub(crate) unanswered: u32,
}

#[atomic_delegates]
impl ResubscribeGate {
    pub fn is_due(&self, now: Instant) -> bool {}
    pub fn sent(&mut self, now: Instant) {}
    pub fn heal_replica_split(
        &mut self,
        subscription: &mut discovery::LobbyContractSubscription,
        now: Instant,
    ) {
    }
}

#[rustfmt::skip]
impl ResubscribeGate {
    pub fn answered(&mut self) -> u32 { std::mem::take(&mut self.unanswered) }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::Instant;

    use super::ResubscribeGate;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let mut gate = ResubscribeGate::default();
        let started = Instant::now();
        assert!(gate.is_due(started));
        assert_eq!(gate.answered(), 0);
        gate.sent(started);
        assert!(!gate.is_due(started + discovery::RESUBSCRIBE - Duration::from_millis(1)));
        assert!(gate.is_due(started + discovery::RESUBSCRIBE));
        assert_eq!(gate.answered(), 1);
        assert_eq!(gate.answered(), 0);
        assert!(!gate.is_due(started + discovery::RESUBSCRIBE - Duration::from_millis(1)));
        assert!(gate.is_due(started + discovery::RESUBSCRIBE));
    }

    #[tokio::test]
    async fn unanswered_sends_double_the_wait_up_to_the_cap() {
        let mut gate = ResubscribeGate::default();
        let started = Instant::now();
        let mut sent_at = started;
        let mut expected_wait = discovery::RESUBSCRIBE;
        for _ in 0..8 {
            assert!(gate.is_due(sent_at));
            gate.sent(sent_at);
            assert!(!gate.is_due(sent_at + expected_wait - Duration::from_millis(1)));
            assert!(gate.is_due(sent_at + expected_wait));
            sent_at += expected_wait;
            expected_wait = (expected_wait * 2).min(discovery::RESUBSCRIBE_BACKOFF_CAP);
        }
        assert_eq!(expected_wait, discovery::RESUBSCRIBE_BACKOFF_CAP);
        assert_eq!(gate.answered(), 8);
    }

    #[tokio::test]
    async fn an_answer_resets_the_ladder_to_the_base_wait() {
        let mut gate = ResubscribeGate::default();
        let started = Instant::now();
        for _ in 0..4 {
            gate.sent(started);
        }
        assert_eq!(gate.answered(), 4);
        let after_answer = started + Duration::from_secs(37);
        gate.sent(after_answer);
        assert!(!gate.is_due(after_answer + discovery::RESUBSCRIBE - Duration::from_millis(1)));
        assert!(gate.is_due(after_answer + discovery::RESUBSCRIBE));
    }
}
