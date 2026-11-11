use crate::discovery;

pub const fn answered(gate: &mut discovery::ResubscribeGate) -> u32 {
    let missed_sends = gate.unanswered;
    gate.unanswered = 0;
    missed_sends
}

#[cfg(test)]
mod tests {
    use super::answered;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut gate = discovery::ResubscribeGate::default();
        assert_eq!(answered(&mut gate), 0);
        gate.sent(tokio::time::Instant::now());
        gate.sent(tokio::time::Instant::now());
        assert_eq!(answered(&mut gate), 2);
        assert_eq!(answered(&mut gate), 0);
    }
}
