use tokio::time::Instant;

use crate::discovery;

const DOUBLING: u32 = 2;

pub fn sent(gate: &mut discovery::ResubscribeGate, now: Instant) {
    let mut delay = discovery::RESUBSCRIBE;
    let mut remaining_doublings = gate.unanswered;
    while remaining_doublings > 0 {
        delay = delay
            .checked_mul(DOUBLING)
            .unwrap_or(discovery::RESUBSCRIBE_BACKOFF_CAP);
        remaining_doublings = remaining_doublings.saturating_sub(1);
    }
    let capped_delay = delay.min(discovery::RESUBSCRIBE_BACKOFF_CAP);
    gate.next_due = now.checked_add(capped_delay);
    gate.unanswered = gate.unanswered.saturating_add(1);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::Instant;

    use super::sent;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut gate = discovery::ResubscribeGate::default();
        let started = Instant::now();
        sent(&mut gate, started);
        assert!(!gate.is_due(started + discovery::RESUBSCRIBE - Duration::from_millis(1)));
        assert!(gate.is_due(started + discovery::RESUBSCRIBE));
        sent(&mut gate, started + discovery::RESUBSCRIBE);
        assert!(gate.is_due(started + discovery::RESUBSCRIBE * 3));
    }
}
