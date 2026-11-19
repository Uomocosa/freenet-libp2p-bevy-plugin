use std::time::{Duration, Instant};

use crate::testing;

pub struct CachedStatuses {
    pub at: Instant,
    pub statuses: Vec<testing::harness::BoardStatus>,
}

#[rustfmt::skip]
impl CachedStatuses {
    #[must_use]
    pub fn is_fresh(&self, ttl: Duration) -> bool { self.at.elapsed() < ttl }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::CachedStatuses;

    #[test]
    fn test_usage() {
        let cached = CachedStatuses {
            at: Instant::now(),
            statuses: Vec::new(),
        };
        assert!(cached.statuses.is_empty());
        assert!(cached.is_fresh(Duration::from_secs(10)));
        assert!(!cached.is_fresh(Duration::ZERO));
    }
}
