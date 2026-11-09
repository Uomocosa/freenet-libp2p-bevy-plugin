use std::time::Duration;

use crate::UnixTime;

pub fn checked_add(current: UnixTime, by: Duration) -> Option<UnixTime> {
    let extra = u64::try_from(by.as_millis()).ok()?;
    current
        .as_millis()
        .checked_add(extra)
        .map(UnixTime::from_millis)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::checked_add;
    use crate::UnixTime;

    #[test]
    fn test_usage() {
        assert_eq!(
            checked_add(UnixTime::EPOCH, Duration::from_secs(5)),
            Some(UnixTime::from_secs(5))
        );
        assert_eq!(
            checked_add(UnixTime::from_millis(u64::MAX), Duration::from_millis(1)),
            None
        );
    }
}
