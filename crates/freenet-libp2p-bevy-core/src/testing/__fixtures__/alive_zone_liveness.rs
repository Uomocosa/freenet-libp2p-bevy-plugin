use crate::replica_sync;

#[must_use]
pub const fn alive_zone_liveness() -> replica_sync::state_machine::ZoneLiveness {
    replica_sync::state_machine::ZoneLiveness {
        is_zone_still_listed: true,
        has_other_history_holders: true,
    }
}

#[cfg(test)]
mod tests {
    use super::alive_zone_liveness;

    #[test]
    fn test_usage() {
        let liveness = alive_zone_liveness();
        assert!(liveness.is_zone_still_listed);
        assert!(liveness.has_other_history_holders);
    }
}
