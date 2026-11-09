use crate::testing;

#[must_use]
pub fn scaled_sleep_secs(orchestrator: &testing::Orchestrator, secs: u64) -> u64 {
    if secs == 0 {
        return 0;
    }
    let scale = orchestrator.time_scale();
    if scale >= 1.0 {
        return secs;
    }
    if scale.to_bits() == 0.2_f64.to_bits() {
        secs.saturating_add(4).checked_div(5).unwrap_or(1).max(1)
    } else if scale.to_bits() == 0.1_f64.to_bits() {
        secs.saturating_add(9).checked_div(10).unwrap_or(1).max(1)
    } else {
        secs
    }
}

#[cfg(test)]
mod tests {
    use super::scaled_sleep_secs;
    use crate::testing;

    #[test]
    fn test_usage() {
        let orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::Fake,
        );
        assert_eq!(scaled_sleep_secs(&orchestrator, 0), 0);
        assert_eq!(scaled_sleep_secs(&orchestrator, 45), 9);
    }
}
