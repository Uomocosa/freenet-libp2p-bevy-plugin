use std::time::{SystemTime, UNIX_EPOCH};

use crate::testing;

pub fn now() -> u64 {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis());
    elapsed
        .checked_rem(u128::from(testing::harness::MILLIS_PER_DAY))
        .and_then(|millis| u64::try_from(millis).ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::now;
    use crate::testing;

    #[test]
    fn test_usage() {
        assert!(now() < testing::harness::MILLIS_PER_DAY);
    }
}
