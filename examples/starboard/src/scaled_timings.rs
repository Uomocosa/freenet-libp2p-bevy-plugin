use std::time::Duration;

use freenet_libp2p_bevy_plugin::{discovery, room_sync};

use crate::time_scale;

/// The lease and history timings of one run, scaled by `E2E_TIME_SCALE`.
///
/// The scale exists for the slow Windows peer: CROSS-OS-PLAN X9 multiplies every
/// wait by it, and the lease is the wait a holder's history lives for.
pub fn scaled_lease_timings(time_scale: time_scale::Scale) -> discovery::HolderHeartbeatIntervals {
    discovery::HolderHeartbeatIntervals {
        interval_between_heartbeats: time_scale.scaled(Duration::from_secs(30)),
        ttl: time_scale.scaled(Duration::from_secs(90)),
        remote_clock_skew_margin: time_scale.scaled(Duration::from_secs(600)),
    }
}

/// The history timings of one run, scaled by `E2E_TIME_SCALE`.
///
/// The record lives ten minutes: a peer that needs a mainnet lobby round trip
/// (the contract's state can take minutes to reach a cold node) still finds the
/// board after the holders left the room. The room's death is decided by the
/// lobby, not by the record.
pub fn scaled_history_config(time_scale: time_scale::Scale) -> room_sync::HistoryConfig {
    room_sync::HistoryConfig {
        write_interval: time_scale.scaled(Duration::from_secs(30)),
        record_ttl: time_scale.scaled(Duration::from_secs(600)),
        ..room_sync::HistoryConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{scaled_history_config, scaled_lease_timings};
    use crate::time_scale::Scale;

    #[test]
    fn test_usage() {
        let lease = scaled_lease_timings(Scale(1.0));
        assert_eq!(lease.interval_between_heartbeats, Duration::from_secs(30));
        assert_eq!(lease.ttl, Duration::from_secs(90));
        assert_eq!(lease.remote_clock_skew_margin, Duration::from_secs(600));
        let history = scaled_history_config(Scale(1.0));
        assert_eq!(history.write_interval, Duration::from_secs(30));
        assert_eq!(history.record_ttl, Duration::from_secs(600));
    }

    #[test]
    fn a_scale_of_two_doubles_the_timings() {
        let lease = scaled_lease_timings(Scale(2.0));
        assert_eq!(lease.ttl, Duration::from_secs(180));
        assert_eq!(lease.remote_clock_skew_margin, Duration::from_mins(20));
        let history = scaled_history_config(Scale(2.0));
        assert_eq!(history.write_interval, Duration::from_secs(60));
        assert_eq!(history.record_ttl, Duration::from_mins(20));
    }

    #[test]
    fn a_scale_of_zero_never_waits() {
        let lease = scaled_lease_timings(Scale(0.0));
        assert_eq!(lease.ttl, Duration::ZERO);
        assert_eq!(lease.remote_clock_skew_margin, Duration::ZERO);
    }
}
