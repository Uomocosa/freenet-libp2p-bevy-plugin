use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepWithDeadline<Step> {
    pub step: Step,
    pub within: Option<Duration>,
}

impl<Step> From<Step> for StepWithDeadline<Step> {
    fn from(step: Step) -> Self {
        Self { step, within: None }
    }
}

impl<Step> StepWithDeadline<Step> {
    #[must_use]
    pub const fn within(mut self, duration: Duration) -> Self {
        self.within = Some(duration);
        self
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::StepWithDeadline;

    #[test]
    fn test_usage() {
        let pending = StepWithDeadline::from(7_u32);
        assert_eq!(pending.step, 7_u32);
        assert_eq!(pending.within, None);
        let timed = pending.within(Duration::from_secs(9));
        assert_eq!(timed.within, Some(Duration::from_secs(9)));
    }
}
