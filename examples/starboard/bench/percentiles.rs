use std::time::Duration;

use crate::starboard;

#[must_use]
pub fn percentiles(samples: &[Duration]) -> starboard::bench::LatencyPercentiles {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    starboard::bench::LatencyPercentiles {
        p50: percentile(&sorted, 50),
        p99: percentile(&sorted, 99),
        samples: sorted.len(),
    }
}

// needed helper: the nearest-rank sample at the given percentile
fn percentile(sorted: &[Duration], percent: u32) -> Duration {
    let len = u32::try_from(sorted.len()).unwrap_or(0);
    let rank = len.saturating_mul(percent).saturating_add(99) / 100;
    let index = usize::try_from(rank.saturating_sub(1)).unwrap_or(0);
    sorted.get(index).copied().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::percentiles;

    #[test]
    fn test_usage() {
        let empty = percentiles(&[]);
        assert_eq!(empty.p50, Duration::ZERO);
        assert_eq!(empty.samples, 0);

        let hundred: Vec<Duration> = (1_u64..=100).map(Duration::from_millis).collect();
        let measured = percentiles(&hundred);
        assert_eq!(measured.samples, 100);
        assert_eq!(measured.p50, Duration::from_millis(50));
        assert_eq!(measured.p99, Duration::from_millis(99));

        let single = percentiles(&[Duration::from_millis(7)]);
        assert_eq!(single.p50, Duration::from_millis(7));
        assert_eq!(single.p99, Duration::from_millis(7));
    }
}
