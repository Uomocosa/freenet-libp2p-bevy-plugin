use crate::UnixTime;

#[must_use]
pub fn now() -> UnixTime {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default();
    UnixTime::from_millis(u64::try_from(millis).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::now;
    use crate::UnixTime;

    #[test]
    fn test_usage() {
        assert!(now() > UnixTime::from_secs(1_700_000_000));
    }
}
