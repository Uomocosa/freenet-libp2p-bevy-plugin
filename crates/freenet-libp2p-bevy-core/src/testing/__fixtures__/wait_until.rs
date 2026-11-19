use std::time::Duration;

#[must_use]
pub fn wait_until(rounds: usize, settle: Duration, mut attempt: impl FnMut(usize) -> bool) -> bool {
    for round in 0..rounds {
        if attempt(round) {
            return true;
        }
        std::thread::sleep(settle);
    }
    false
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::wait_until;

    #[test]
    fn test_usage() {
        assert!(wait_until(3, Duration::ZERO, |round| round == 2));
        assert!(!wait_until(2, Duration::ZERO, |round| round == 5));
    }
}
