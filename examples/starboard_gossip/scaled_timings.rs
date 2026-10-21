use std::time::Duration;

use freenet_libp2p_bevy_plugin::{discovery, room_sync};

/// The lease and history timings of one run, scaled by `E2E_TIME_SCALE`.
///
/// The scale exists for the slow Windows peer: CROSS-OS-PLAN X9 multiplies every
/// wait by it, and the lease is the wait a holder's history lives for.
pub fn scaled_lease_timings(time_scale: f64) -> discovery::LeaseTimings {
    discovery::LeaseTimings {
        renew_every: scaled(Duration::from_secs(30), time_scale),
        ttl: scaled(Duration::from_secs(90), time_scale),
    }
}

/// The history timings of one run, scaled by `E2E_TIME_SCALE`.
///
/// The record lives ten minutes: a peer that needs a mainnet lobby round trip
/// (the contract's state can take minutes to reach a cold node) still finds the
/// board after the holders left the room. The room's death is decided by the
/// lobby, not by the record.
pub fn scaled_history_config(time_scale: f64) -> room_sync::HistoryConfig {
    room_sync::HistoryConfig {
        write_interval: scaled(Duration::from_secs(30), time_scale),
        record_ttl: scaled(Duration::from_secs(600), time_scale),
        ..room_sync::HistoryConfig::default()
    }
}

// needed helper: one duration scaled by the run's time scale
fn scaled(duration: Duration, time_scale: f64) -> Duration {
    let seconds = duration.as_secs_f64() * time_scale.max(0.0);
    if seconds <= 0.0 {
        return Duration::ZERO;
    }
    Duration::from_secs_f64(seconds)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{scaled_history_config, scaled_lease_timings};

    #[test]
    fn test_usage() {
        let lease = scaled_lease_timings(1.0);
        assert_eq!(lease.renew_every, Duration::from_secs(30));
        assert_eq!(lease.ttl, Duration::from_secs(90));
        let history = scaled_history_config(1.0);
        assert_eq!(history.write_interval, Duration::from_secs(30));
        assert_eq!(history.record_ttl, Duration::from_secs(600));
    }

    #[test]
    fn a_scale_of_two_doubles_the_timings() {
        let lease = scaled_lease_timings(2.0);
        assert_eq!(lease.ttl, Duration::from_secs(180));
        let history = scaled_history_config(2.0);
        assert_eq!(history.write_interval, Duration::from_secs(60));
        assert_eq!(history.record_ttl, Duration::from_secs(1200));
    }

    #[test]
    fn a_scale_of_zero_never_waits() {
        let lease = scaled_lease_timings(0.0);
        assert_eq!(lease.ttl, Duration::ZERO);
    }
}
