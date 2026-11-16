use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use derive_more::Deref;

use crate::testing;

#[derive(Debug, Clone, Copy, PartialEq, Deref)]
pub struct Deadlines(pub f64);

impl From<testing::Network> for Deadlines {
    fn from(network: testing::Network) -> Self {
        let default = match network {
            testing::Network::Fake => 1.0,
            testing::Network::Mainnet
            | testing::Network::LocalGateways
            | testing::Network::DirectDial => testing::harness::REAL_NETWORK_MULTIPLIER,
        };
        Self(Self::scale_from_env().unwrap_or(default))
    }
}

#[rustfmt::skip]
impl Deadlines {
    #[must_use]
    pub fn scaled(self, base: Duration) -> Duration { base.mul_f64(*self) }
    #[must_use]
    pub fn action(self) -> Duration { self.scaled(testing::harness::FAKE_ACTION) }
    #[must_use]
    pub fn lobby(self) -> Duration { self.scaled(testing::harness::FAKE_LOBBY) }
}

#[atomic_delegates]
impl Deadlines {
    pub(crate) fn scale_from_env() -> Option<f64> {}
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Deadlines;
    use crate::testing;

    #[test]
    fn test_usage() {
        let fake = Deadlines(1.0);
        assert_eq!(fake.lobby(), Duration::from_secs(120));
        assert_eq!(fake.action(), Duration::from_secs(60));
        let real = Deadlines::from(testing::Network::Mainnet);
        let scaled = real.lobby();
        assert!(scaled >= Duration::from_secs(400));
        assert!(Deadlines::from(testing::Network::Fake).action() <= Duration::from_secs(60) * 2);
    }
}
