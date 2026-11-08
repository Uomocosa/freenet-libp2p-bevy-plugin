use freenet_libp2p_bevy_plugin::time;

use crate::starboard;

#[must_use]
pub fn day_index(created_at: time::UnixTime, now: time::UnixTime) -> u64 {
    now.elapsed_since(created_at)
        .as_secs()
        .checked_div(starboard::identity::SECONDS_PER_DAY)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::time::UnixTime;

    use super::day_index;

    #[test]
    fn test_usage() {
        let created_at = UnixTime::from(1_000);
        assert_eq!(day_index(created_at, UnixTime::from(500)), 0);
        assert_eq!(day_index(created_at, UnixTime::from(1_000)), 0);
        assert_eq!(day_index(created_at, UnixTime::from(87_399)), 0);
        assert_eq!(day_index(created_at, UnixTime::from(87_400)), 1);
        assert_eq!(day_index(created_at, UnixTime::from(173_800)), 2);
    }
}
