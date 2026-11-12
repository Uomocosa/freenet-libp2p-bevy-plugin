use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LatencyPercentiles {
    pub p50: Duration,
    pub p99: Duration,
    pub samples: usize,
}

impl LatencyPercentiles {
    pub(crate) fn nearest_rank(sorted: &[Duration], percent: u32) -> Duration {
        let len = u32::try_from(sorted.len()).unwrap_or(0);
        let rank = len.saturating_mul(percent).saturating_add(99) / 100;
        let index = usize::try_from(rank.saturating_sub(1)).unwrap_or(0);
        sorted.get(index).copied().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::LatencyPercentiles;

    #[test]
    fn test_usage() {
        let sorted = [1, 2, 3].map(Duration::from_millis);
        assert_eq!(
            LatencyPercentiles::nearest_rank(&sorted, 50),
            Duration::from_millis(2)
        );
        assert_eq!(LatencyPercentiles::nearest_rank(&[], 99), Duration::ZERO);
    }
}
