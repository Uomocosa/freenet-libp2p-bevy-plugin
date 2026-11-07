use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::*;

use crate::testing;

#[derive(Resource, Debug, Clone)]
pub struct ScriptRunner<Step> {
    pub steps: Vec<testing::TimedScriptStep<Step>>,
    pub index: usize,
    pub step_elapsed: Duration,
    pub is_first_poll: bool,
    pub verdict: testing::ScriptVerdict,
    pub finished_step_timings: Vec<testing::StepTiming>,
}

impl<Step> ScriptRunner<Step> {
    #[must_use]
    pub const fn new(steps: Vec<testing::TimedScriptStep<Step>>) -> Self {
        Self {
            steps,
            index: 0,
            step_elapsed: Duration::ZERO,
            is_first_poll: true,
            verdict: testing::ScriptVerdict::Running,
            finished_step_timings: Vec::new(),
        }
    }
}

#[atomic_delegates]
impl<Step: testing::ScriptStep> ScriptRunner<Step> {
    pub fn advance(
        &mut self,
        delta: Duration,
        poll_step: impl FnMut(&Step, testing::StepClock) -> testing::StepOutcome,
    ) -> testing::ScriptVerdict {
    }
}

#[atomic_delegates]
impl<Step: testing::ScriptStep> ScriptRunner<Step> {
    pub(crate) fn fail(
        &mut self,
        position: &str,
        raw: &str,
        reason: &str,
    ) -> testing::ScriptVerdict {
    }
}

#[cfg(test)]
mod tests {
    use super::ScriptRunner;
    use crate::testing;
    #[test]
    fn test_usage() {
        let runner = ScriptRunner::<u8>::new(Vec::new());
        assert_eq!(runner.index, 0);
        assert_eq!(runner.verdict, testing::ScriptVerdict::Running);
        assert_eq!(runner.finished_step_timings.len(), 0);
    }
}
