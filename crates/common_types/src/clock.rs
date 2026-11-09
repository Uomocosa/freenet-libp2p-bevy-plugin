use crate::UnixTime;

impl lele_bevy_state_machine::Clock for UnixTime {
    fn now() -> Self {
        Self::now()
    }
}

#[cfg(test)]
mod tests {
    use super::UnixTime;
    use lele_bevy_state_machine::Clock;

    #[test]
    fn test_usage() {
        assert!(UnixTime::now() > UnixTime::from_secs(1_700_000_000));
        assert!(<UnixTime as Clock>::now() > UnixTime::from_secs(1_700_000_000));
    }
}
