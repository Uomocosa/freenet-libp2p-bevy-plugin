use std::time::Duration;

use crate::testing;

pub fn advance<Step: testing::traits::ScriptStep>(
    runner: &mut testing::ScriptRunner<Step>,
    delta: Duration,
    mut poll_step: impl FnMut(&Step, testing::StepClock) -> testing::StepOutcome,
) -> testing::ScriptVerdict {
    if runner.verdict != testing::ScriptVerdict::Running {
        return runner.verdict;
    }
    runner.step_elapsed = runner.step_elapsed.saturating_add(delta);
    loop {
        let Some(timed) = runner.steps.get(runner.index).cloned() else {
            tracing::info!(target: "script", "script done steps={}", runner.steps.len());
            runner.verdict = testing::ScriptVerdict::Finished;
            return runner.verdict;
        };
        let clock = testing::StepClock {
            elapsed: runner.step_elapsed,
            is_first_poll: runner.is_first_poll,
        };
        runner.is_first_poll = false;
        let position = format!("{}/{}", runner.index.saturating_add(1), runner.steps.len());
        match poll_step(&timed.step, clock) {
            testing::StepOutcome::Done => {
                tracing::info!(target: "script", "script step done step={position} {}", timed.raw);
                runner.finished_step_timings.push(testing::StepTiming {
                    raw: timed.raw.clone(),
                    secs: runner.step_elapsed.as_secs_f64(),
                    failure: None,
                });
                runner.index = runner.index.saturating_add(1);
                runner.step_elapsed = Duration::ZERO;
                runner.is_first_poll = true;
            }
            testing::StepOutcome::Failed(reason) => {
                return runner.fail(&position, &timed.raw, &reason);
            }
            testing::StepOutcome::Pending => {
                if let Some(deadline) = timed.deadline
                    && runner.step_elapsed > deadline
                {
                    let reason = format!("timed out after {}s", deadline.as_secs());
                    return runner.fail(&position, &timed.raw, &reason);
                }
                return testing::ScriptVerdict::Running;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;

    use super::advance;
    use crate::testing;

    fn timed(
        step: testing::E2EStep,
        raw: &str,
        deadline: Option<u64>,
    ) -> testing::TimedScriptStep<testing::E2EStep> {
        testing::TimedScriptStep {
            step,
            deadline: deadline.map(Duration::from_secs),
            raw: raw.to_string(),
        }
    }

    fn runner(
        steps: Vec<testing::TimedScriptStep<testing::E2EStep>>,
    ) -> testing::ScriptRunner<testing::E2EStep> {
        testing::ScriptRunner::new(steps)
    }

    #[test]
    fn test_usage() {
        let mut script = runner(vec![
            timed(testing::E2EStep::Stay, "step-0", None),
            timed(testing::E2EStep::Stay, "step-1", None),
        ]);
        let verdict = advance(&mut script, Duration::ZERO, |_, _| {
            testing::StepOutcome::Done
        });
        assert_eq!(verdict, testing::ScriptVerdict::Finished);
        assert_eq!(
            testing::ScriptVerdict::Finished.app_exit_for_verdict(),
            Some(AppExit::Success)
        );
        assert_eq!(script.finished_step_timings.len(), 2);
        assert!(
            script
                .finished_step_timings
                .iter()
                .all(|timing| timing.failure.is_none())
        );
    }

    #[test]
    fn a_wait_past_its_deadline_fails_the_script() {
        let mut script = runner(vec![timed(testing::E2EStep::Stay, "step-0", Some(5))]);
        let pending = |_: &testing::E2EStep, _| testing::StepOutcome::Pending;
        assert_eq!(
            advance(&mut script, Duration::from_secs(5), pending),
            testing::ScriptVerdict::Running
        );
        assert_eq!(
            advance(&mut script, Duration::from_secs(1), pending),
            testing::ScriptVerdict::Failed
        );
        assert_eq!(
            advance(&mut script, Duration::ZERO, |_, _| {
                testing::StepOutcome::Done
            }),
            testing::ScriptVerdict::Failed
        );
        assert_eq!(
            testing::ScriptVerdict::Failed.app_exit_for_verdict(),
            Some(AppExit::from_code(testing::SCRIPT_FAILED_EXIT_CODE))
        );
    }

    #[test]
    fn each_step_gets_a_fresh_clock() {
        let mut script = runner(vec![
            timed(testing::E2EStep::Stay, "step-0", None),
            timed(testing::E2EStep::ExitApp, "step-1", None),
        ]);
        let mut seen = Vec::new();
        advance(&mut script, Duration::from_secs(9), |step, clock| {
            seen.push((step.clone(), clock.elapsed, clock.is_first_poll));
            if matches!(step, testing::E2EStep::Stay) {
                testing::StepOutcome::Done
            } else {
                testing::StepOutcome::Pending
            }
        });
        assert_eq!(
            seen,
            vec![
                (testing::E2EStep::Stay, Duration::from_secs(9), true),
                (testing::E2EStep::ExitApp, Duration::ZERO, true),
            ]
        );
    }

    #[test]
    fn finished_steps_record_their_timings() {
        let mut script = runner(vec![
            timed(testing::E2EStep::Stay, "step-0", None),
            timed(testing::E2EStep::ExitApp, "step-1", None),
        ]);
        advance(&mut script, Duration::from_secs(2), |step, _| {
            if matches!(step, testing::E2EStep::Stay) {
                testing::StepOutcome::Done
            } else {
                testing::StepOutcome::Failed("boom".to_string())
            }
        });
        assert_eq!(script.finished_step_timings.len(), 2);
        assert_eq!(script.finished_step_timings[0].raw, "step-0");
        assert!(script.finished_step_timings[0].failure.is_none());
        assert_eq!(
            script.finished_step_timings[1].failure.as_deref(),
            Some("boom")
        );
    }
}
