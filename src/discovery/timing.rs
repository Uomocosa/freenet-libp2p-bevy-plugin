use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    pub tick: Duration,
    pub republish: Duration,
    pub presence_ttl: Duration,
    pub lobby: Duration,
    pub redial: Duration,
    pub hello: Duration,
    pub member_grace: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            tick: Duration::from_secs(1),
            republish: Duration::from_secs(30),
            presence_ttl: Duration::from_secs(120),
            lobby: Duration::from_secs(5),
            redial: Duration::from_secs(2),
            hello: Duration::from_secs(5),
            member_grace: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Timing;

    #[test]
    fn test_usage() {
        let timing = Timing::default();
        assert_eq!(timing.tick, Duration::from_secs(1));
        assert!(timing.republish > timing.tick);
    }
}
