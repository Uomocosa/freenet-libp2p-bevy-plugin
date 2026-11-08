use std::time::Duration;

use crate::freenet;
use freenet::basic::constants;

const JITTER_SPAN: u64 = 41;
const JITTER_BASE: u64 = 80;
const JITTER_DIVISOR: u128 = 100;
const DOUBLE: u32 = 2;
const MAX_SHIFT: u32 = 10;

#[must_use]
pub fn reconnect_delay(failures: u32, random: u64) -> Duration {
    let capped_shift = failures.saturating_sub(1).min(MAX_SHIFT);
    let mut base = constants::RECONNECT_INITIAL;
    let mut remaining = capped_shift;
    while remaining > 0 {
        remaining = remaining.saturating_sub(1);
        base = base.checked_mul(DOUBLE).unwrap_or(constants::RECONNECT_MAX);
        if base >= constants::RECONNECT_MAX {
            base = constants::RECONNECT_MAX;
            break;
        }
    }
    if base >= constants::RECONNECT_MAX {
        base = constants::RECONNECT_MAX;
    }
    let remainder = random.checked_rem(JITTER_SPAN).unwrap_or(0);
    let percent = remainder.checked_add(JITTER_BASE).unwrap_or(JITTER_BASE);
    let millis = base.as_millis();
    let scaled = millis.checked_mul(u128::from(percent)).unwrap_or(millis);
    let divided = scaled.checked_div(JITTER_DIVISOR).unwrap_or(millis);
    let capped = divided.min(constants::RECONNECT_MAX.as_millis());
    let millis_u64 = u64::try_from(capped).unwrap_or(u64::MAX);
    Duration::from_millis(millis_u64)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::reconnect_delay;
    use crate::freenet;
    use freenet::basic::constants;

    #[test]
    fn test_usage() {
        let delay = reconnect_delay(1, 0);
        assert!(delay >= Duration::from_millis(2_400));
        assert!(delay <= constants::RECONNECT_MAX);
    }

    #[test]
    fn first_failure_spans_initial_with_jitter() {
        assert_eq!(reconnect_delay(1, 0), Duration::from_millis(2_400));
        assert_eq!(reconnect_delay(1, 40), Duration::from_millis(3_600));
    }

    #[test]
    fn second_failure_doubles_before_jitter() {
        assert_eq!(reconnect_delay(2, 0), Duration::from_millis(4_800));
        assert_eq!(reconnect_delay(2, 40), Duration::from_millis(7_200));
    }

    #[test]
    fn large_failures_stay_capped() {
        assert_eq!(reconnect_delay(100, 0), Duration::from_secs(48));
        assert_eq!(reconnect_delay(100, 40), constants::RECONNECT_MAX);
    }
}
