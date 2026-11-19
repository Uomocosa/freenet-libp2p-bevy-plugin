use std::time::Duration;

use crate::discovery;

#[must_use]
pub const fn lease_limits(
    lease_ttl: Duration,
    remote_clock_skew_margin: Duration,
) -> discovery::LobbyLeaseLimits {
    discovery::LobbyLeaseLimits {
        lease_ttl,
        remote_clock_skew_margin,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::lease_limits;

    #[test]
    fn test_usage() {
        let limits = lease_limits(Duration::from_secs(90), Duration::from_secs(600));
        assert_eq!(limits.lease_ttl, Duration::from_secs(90));
        assert_eq!(limits.remote_clock_skew_margin, Duration::from_secs(600));
    }
}
