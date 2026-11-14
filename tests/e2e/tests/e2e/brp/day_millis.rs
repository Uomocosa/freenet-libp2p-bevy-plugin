use std::time::{SystemTime, UNIX_EPOCH};

const MILLIS_PER_DAY: u64 = 86_400_000;

/// The current UTC time of day in milliseconds, as the apps' log timestamps read it.
#[must_use]
pub fn day_millis_now() -> u64 {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis());
    elapsed
        .checked_rem(u128::from(MILLIS_PER_DAY))
        .and_then(|millis| u64::try_from(millis).ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{MILLIS_PER_DAY, day_millis_now};

    #[test]
    fn test_usage() {
        assert!(day_millis_now() < MILLIS_PER_DAY);
    }
}
