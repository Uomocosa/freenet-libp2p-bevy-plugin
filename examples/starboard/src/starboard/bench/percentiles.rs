use std::time::Duration;

use crate::starboard;
use crate::starboard::bench::LatencyPercentiles;

#[must_use]
pub fn percentiles(samples: &[Duration]) -> starboard::bench::LatencyPercentiles {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    starboard::bench::LatencyPercentiles {
        p50: LatencyPercentiles::nearest_rank(&sorted, 50),
        p99: LatencyPercentiles::nearest_rank(&sorted, 99),
        samples: sorted.len(),
    }
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
