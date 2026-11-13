use crate::UnixTime;

pub fn saturating_add(time: UnixTime, by: std::time::Duration) -> UnixTime {
    UnixTime::from_millis(
        time.as_millis()
            .saturating_add(u64::try_from(by.as_millis()).unwrap_or(u64::MAX)),
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::saturating_add;
    use crate::UnixTime;

    #[test]
    fn test_usage() {
        assert_eq!(
            saturating_add(UnixTime::from_secs(1), Duration::from_millis(500)),
            UnixTime::from_millis(1500)
        );
        assert_eq!(
            saturating_add(UnixTime::from_millis(u64::MAX), Duration::from_secs(1)),
            UnixTime::from_millis(u64::MAX)
        );
    }
}
