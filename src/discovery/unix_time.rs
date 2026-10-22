use std::time::Duration;

use bevy::prelude::Reflect;
use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref, Reflect,
)]
pub struct UnixTime(u64);

#[rustfmt::skip]
impl From<u64> for UnixTime {
    fn from(secs: u64) -> Self { Self(secs) }
}

impl UnixTime {
    #[must_use]
    pub fn now() -> Self {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();
        Self::from(secs)
    }
}

#[rustfmt::skip]
impl UnixTime {
    #[must_use]
    pub fn as_secs(self) -> u64 { *self }

    #[must_use]
    pub fn elapsed_since(self, earlier: Self) -> Duration {
        let secs = self.as_secs().saturating_sub(earlier.as_secs());
        Duration::from_secs(secs)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::UnixTime;

    #[test]
    fn test_usage() {
        assert_eq!(UnixTime::from(60).as_secs(), 60);
        assert_eq!(
            UnixTime::from(25).elapsed_since(UnixTime::from(10)),
            Duration::from_secs(15)
        );
        assert!(UnixTime::now() > UnixTime::from(1_700_000_000));
    }

    #[test]
    fn wire_is_transparent() {
        let time = UnixTime::from(9);
        assert_eq!(
            bincode::serialize(&time).unwrap_or_default(),
            bincode::serialize(&9_u64).unwrap_or_default()
        );
    }
}
