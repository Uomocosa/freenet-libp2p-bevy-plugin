use tokio::time::Instant;

use crate::discovery;

pub fn is_due(gate: &discovery::ResubscribeGate, now: Instant) -> bool {
    let never_sent = gate.next_due.is_none();
    never_sent || gate.next_due.is_some_and(|next_due| now >= next_due)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::Instant;

    use super::is_due;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut gate = discovery::ResubscribeGate::default();
        let started = Instant::now();
        assert!(is_due(&gate, started));
        gate.sent(started);
        assert!(!is_due(
            &gate,
            started + discovery::RESUBSCRIBE - Duration::from_millis(1)
        ));
        assert!(is_due(&gate, started + discovery::RESUBSCRIBE));
    }
}
