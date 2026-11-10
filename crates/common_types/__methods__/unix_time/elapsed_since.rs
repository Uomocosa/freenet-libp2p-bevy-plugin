use std::time::Duration;

use crate::UnixTime;

#[must_use]
pub const fn elapsed_since(current: UnixTime, earlier: UnixTime) -> Duration {
    let millis = current.as_millis().saturating_sub(earlier.as_millis());
    Duration::from_millis(millis)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::elapsed_since;
    use crate::UnixTime;

    #[test]
    fn test_usage() {
        assert_eq!(
            elapsed_since(UnixTime::from_secs(25), UnixTime::from_secs(10)),
            Duration::from_secs(15)
        );
        assert_eq!(
            elapsed_since(UnixTime::from_secs(10), UnixTime::from_secs(25)),
            Duration::from_secs(0)
        );
    }
}
