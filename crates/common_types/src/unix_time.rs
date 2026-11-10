use atomic_delegate_macros::atomic_delegates;
use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref,
)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
pub struct UnixTime(u64);

impl UnixTime {
    pub const EPOCH: Self = Self(0);

    #[must_use]
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs.saturating_mul(1_000))
    }

    #[must_use]
    pub const fn from_mins(mins: u64) -> Self {
        Self::from_secs(mins.saturating_mul(60))
    }

    #[must_use]
    pub const fn from_hours(hours: u64) -> Self {
        Self::from_mins(hours.saturating_mul(60))
    }
}

#[atomic_delegates]
impl UnixTime {
    #[must_use]
    pub fn now() -> Self {}
}

#[cfg(feature = "bevy")]
impl lele_bevy_state_machine::Clock for UnixTime {
    fn now() -> Self {
        Self::now()
    }
}

#[atomic_delegates]
impl UnixTime {
    #[must_use]
    pub fn checked_add(self, by: std::time::Duration) -> Option<Self> {}
    #[must_use]
    pub const fn elapsed_since(self, earlier: Self) -> std::time::Duration {}
}

#[rustfmt::skip]
impl UnixTime {
    #[must_use]
    pub const fn as_millis(self) -> u64 {
        let Self(millis) = self;
        millis
    }

    #[must_use]
    pub fn saturating_add(self, by: std::time::Duration) -> Self {
        Self::from_millis(
            self.as_millis()
                .saturating_add(u64::try_from(by.as_millis()).unwrap_or(u64::MAX)),
        )
    }

    #[must_use]
    pub const fn as_secs(self) -> u64 {
        self.as_millis() / 1_000
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rstest::rstest;

    use super::UnixTime;

    #[test]
    fn test_usage() {
        assert_eq!(UnixTime::from_secs(1), UnixTime::from_millis(1_000));
        assert_eq!(UnixTime::from_mins(1), UnixTime::from_secs(60));
        assert_eq!(UnixTime::from_hours(1), UnixTime::from_mins(60));
        assert_eq!(UnixTime::from_millis(1_999).as_secs(), 1);
        assert_eq!(
            UnixTime::EPOCH.checked_add(Duration::from_secs(5)),
            Some(UnixTime::from_secs(5))
        );
        assert_eq!(
            UnixTime::from_secs(5).saturating_add(Duration::from_secs(10)),
            UnixTime::from_secs(15)
        );
        assert_eq!(
            UnixTime::EPOCH.saturating_add(Duration::from_millis(u64::MAX)),
            UnixTime::from_millis(u64::MAX)
        );
        assert_eq!(
            UnixTime::from_secs(25).elapsed_since(UnixTime::from_secs(10)),
            Duration::from_secs(15)
        );
        assert!(UnixTime::now() > UnixTime::from_secs(1_700_000_000));
    }

    #[rstest]
    #[case::normal(25, 10, 15)]
    #[case::saturates_on_reversed(10, 25, 0)]
    #[case::zero(0, 0, 0)]
    fn elapsed_since_is_saturating(#[case] now: u64, #[case] earlier: u64, #[case] expected: u64) {
        assert_eq!(
            UnixTime::from_secs(now).elapsed_since(UnixTime::from_secs(earlier)),
            Duration::from_secs(expected)
        );
    }

    #[rstest]
    #[case::zero(0, 0)]
    #[case::one_second(1_000, 1)]
    #[case::floors(1_999, 1)]
    fn as_secs_agrees_with_constructors(#[case] millis: u64, #[case] secs: u64) {
        assert_eq!(UnixTime::from_millis(millis).as_secs(), secs);
        assert_eq!(
            UnixTime::from_secs(secs).as_millis(),
            secs.saturating_mul(1_000)
        );
    }

    #[test]
    fn checked_add_returns_none_on_overflow() {
        assert_eq!(
            UnixTime::from_millis(u64::MAX).checked_add(Duration::from_millis(1)),
            None
        );
    }

    #[test]
    fn wire_is_transparent() {
        let time = UnixTime::from_millis(9);
        assert_eq!(
            bincode::serialize(&time).unwrap_or_default(),
            bincode::serialize(&9_u64).unwrap_or_default()
        );
    }

    #[cfg(feature = "bevy")]
    #[test]
    fn clock_now_matches_unix_time_now() {
        use lele_bevy_state_machine::Clock;

        assert!(UnixTime::now() > UnixTime::from_secs(1_700_000_000));
        assert!(<UnixTime as Clock>::now() > UnixTime::from_secs(1_700_000_000));
    }
}
