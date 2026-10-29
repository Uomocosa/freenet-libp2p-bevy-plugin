use std::time::Duration;

use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::e2e_report::StepTiming;

use crate::shared::script::timed_script_step::TimedScriptStep;

/// The exit code of a peer whose script failed: a wait missed its deadline or a step failed.
pub const SCRIPT_FAILED_EXIT_CODE: u8 = 3;

/// How far one step got this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepOutcome {
    Pending,
    Done,
    Failed(String),
}

/// How long the current step has been running, and whether this is its first frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepClock {
    pub elapsed: Duration,
    pub is_first_poll: bool,
}

/// Where the whole script stands after a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptVerdict {
    Running,
    Finished,
    Failed,
}

/// The scripted session and how far it has got. The runner owns every deadline; an
/// example only says, per step, whether it is pending, done or failed.
#[derive(Resource, Debug, Clone)]
pub struct ScriptRunner<Step> {
    pub steps: Vec<TimedScriptStep<Step>>,
    pub index: usize,
    pub step_elapsed: Duration,
    pub is_first_poll: bool,
    pub verdict: ScriptVerdict,
    pub finished_step_timings: Vec<StepTiming>,
}

impl<Step: Clone> ScriptRunner<Step> {
    #[must_use]
    pub const fn new(steps: Vec<TimedScriptStep<Step>>) -> Self {
        Self {
            steps,
            index: 0,
            step_elapsed: Duration::ZERO,
            is_first_poll: true,
            verdict: ScriptVerdict::Running,
            finished_step_timings: Vec::new(),
        }
    }

    /// Runs as many steps as finish this frame, and logs each one, the script's end, or the
    /// step that failed and why.
    pub fn advance(
        &mut self,
        delta: Duration,
        mut poll_step: impl FnMut(&Step, StepClock) -> StepOutcome,
    ) -> ScriptVerdict {
        if self.verdict != ScriptVerdict::Running {
            return self.verdict;
        }
        self.step_elapsed = self.step_elapsed.saturating_add(delta);
        loop {
            let Some(timed) = self.steps.get(self.index).cloned() else {
                tracing::info!("script done steps={}", self.steps.len());
                self.verdict = ScriptVerdict::Finished;
                return self.verdict;
            };
            let clock = StepClock {
                elapsed: self.step_elapsed,
                is_first_poll: self.is_first_poll,
            };
            self.is_first_poll = false;
            let position = format!("{}/{}", self.index.saturating_add(1), self.steps.len());
            match poll_step(&timed.step, clock) {
                StepOutcome::Done => {
                    tracing::info!("script step done step={position} {}", timed.raw);
                    self.finished_step_timings.push(StepTiming {
                        raw: timed.raw.clone(),
                        secs: self.step_elapsed.as_secs_f64(),
                        failure: None,
                    });
                    self.index = self.index.saturating_add(1);
                    self.step_elapsed = Duration::ZERO;
                    self.is_first_poll = true;
                }
                StepOutcome::Failed(reason) => {
                    return self.fail(&position, &timed.raw, &reason);
                }
                StepOutcome::Pending => {
                    if let Some(deadline) = timed.deadline
                        && self.step_elapsed > deadline
                    {
                        let reason = format!("timed out after {}s", deadline.as_secs());
                        return self.fail(&position, &timed.raw, &reason);
                    }
                    return ScriptVerdict::Running;
                }
            }
        }
    }

    // needed helper: logs the failed step once and stops the script
    fn fail(&mut self, position: &str, raw: &str, reason: &str) -> ScriptVerdict {
        tracing::error!("script failed step={position} {raw} reason={reason}");
        self.finished_step_timings.push(StepTiming {
            raw: raw.to_string(),
            secs: self.step_elapsed.as_secs_f64(),
            failure: Some(reason.to_string()),
        });
        self.verdict = ScriptVerdict::Failed;
        self.verdict
    }
}

/// The app exit a script verdict calls for, if it calls for one.
#[must_use]
pub const fn app_exit_for_verdict(verdict: ScriptVerdict) -> Option<AppExit> {
    match verdict {
        ScriptVerdict::Running => None,
        ScriptVerdict::Finished => Some(AppExit::Success),
        ScriptVerdict::Failed => Some(AppExit::from_code(SCRIPT_FAILED_EXIT_CODE)),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;

    use super::{ScriptRunner, ScriptVerdict, StepOutcome, app_exit_for_verdict};
    use crate::shared::script::timed_script_step::TimedScriptStep;

    fn timed(step: u8, deadline: Option<u64>) -> TimedScriptStep<u8> {
        TimedScriptStep {
            step,
            deadline: deadline.map(Duration::from_secs),
            raw: format!("step-{step}"),
        }
    }

    #[test]
    fn test_usage() {
        let mut runner = ScriptRunner::new(vec![timed(0, None), timed(1, None)]);
        let verdict = runner.advance(Duration::ZERO, |_, _| StepOutcome::Done);
        assert_eq!(verdict, ScriptVerdict::Finished);
        assert_eq!(app_exit_for_verdict(verdict), Some(AppExit::Success));
        assert_eq!(runner.finished_step_timings.len(), 2);
        assert!(
            runner
                .finished_step_timings
                .iter()
                .all(|timing| timing.failure.is_none())
        );
    }

    #[test]
    fn a_wait_past_its_deadline_fails_the_script() {
        let mut runner = ScriptRunner::new(vec![timed(0, Some(5))]);
        let pending = |_: &u8, _| StepOutcome::Pending;
        assert_eq!(
            runner.advance(Duration::from_secs(5), pending),
            ScriptVerdict::Running
        );
        assert_eq!(
            runner.advance(Duration::from_secs(1), pending),
            ScriptVerdict::Failed
        );
        assert_eq!(
            runner.advance(Duration::ZERO, |_, _| StepOutcome::Done),
            ScriptVerdict::Failed,
            "a failed script stays failed"
        );
        assert_eq!(
            app_exit_for_verdict(ScriptVerdict::Failed),
            Some(AppExit::from_code(super::SCRIPT_FAILED_EXIT_CODE))
        );
    }

    #[test]
    fn each_step_gets_a_fresh_clock() {
        let mut runner = ScriptRunner::new(vec![timed(0, None), timed(1, None)]);
        let mut seen = Vec::new();
        runner.advance(Duration::from_secs(9), |step, clock| {
            seen.push((*step, clock.elapsed, clock.is_first_poll));
            if *step == 0 {
                StepOutcome::Done
            } else {
                StepOutcome::Pending
            }
        });
        assert_eq!(
            seen,
            vec![(0, Duration::from_secs(9), true), (1, Duration::ZERO, true),]
        );
    }

    #[test]
    fn finished_steps_record_their_timings() {
        let mut runner = ScriptRunner::new(vec![timed(0, None), timed(1, None)]);
        runner.advance(Duration::from_secs(2), |step, _| {
            if *step == 0 {
                StepOutcome::Done
            } else {
                StepOutcome::Failed("boom".to_string())
            }
        });
        assert_eq!(runner.finished_step_timings.len(), 2);
        assert_eq!(runner.finished_step_timings[0].raw, "step-0");
        assert!(runner.finished_step_timings[0].failure.is_none());
        assert_eq!(
            runner.finished_step_timings[1].failure.as_deref(),
            Some("boom")
        );
    }
}
