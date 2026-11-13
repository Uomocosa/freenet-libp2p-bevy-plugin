use std::time::Duration;

use derive_more::Deref;
use serde::{Deserialize, Serialize};

use crate::UnixTime;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Deref,
)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
pub struct Deadline(UnixTime);

impl Deadline {
    #[must_use]
    pub const fn at(time: UnixTime) -> Self {
        Self(time)
    }
    #[must_use]
    pub fn from_now_plus(now: UnixTime, wait: Duration) -> Self {
        Self(now.saturating_add(wait))
    }
}

#[rustfmt::skip]
impl Deadline {
    #[must_use]
    pub fn is_due(self, now: UnixTime) -> bool { now >= *self }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Deadline;
    use crate::UnixTime;

    #[test]
    fn test_usage() {
        let deadline = Deadline::from_now_plus(UnixTime::from_secs(5), Duration::from_secs(10));
        assert_eq!(deadline, Deadline::at(UnixTime::from_secs(15)));
        assert!(!deadline.is_due(UnixTime::from_secs(14)));
        assert!(deadline.is_due(UnixTime::from_secs(15)));
    }

    #[test]
    fn is_due_at_exact_deadline() {
        let deadline = Deadline::at(UnixTime::from_millis(1_000));
        assert!(deadline.is_due(UnixTime::from_millis(1_000)));
        assert!(!deadline.is_due(UnixTime::from_millis(999)));
    }

    #[test]
    fn after_saturates_at_the_far_future() {
        assert_eq!(
            Deadline::from_now_plus(UnixTime::from_millis(u64::MAX), Duration::from_secs(1)),
            Deadline::at(UnixTime::from_millis(u64::MAX))
        );
    }

    #[test]
    fn at_epoch_is_due_immediately() {
        assert!(Deadline::at(UnixTime::EPOCH).is_due(UnixTime::EPOCH));
    }
}
