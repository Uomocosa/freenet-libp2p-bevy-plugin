use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AntiEntropyConfig {
    pub interval: Duration,
    pub max_parts_in_flight: usize,
}

impl Default for AntiEntropyConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(2),
            max_parts_in_flight: 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::AntiEntropyConfig;

    #[test]
    fn test_usage() {
        let config = AntiEntropyConfig::default();
        assert_eq!(config.interval, Duration::from_secs(2));
        assert_eq!(config.max_parts_in_flight, 8);
    }
}
