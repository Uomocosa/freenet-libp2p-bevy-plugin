use crate::testing;

pub fn fail<Step: testing::ScriptStep>(
    runner: &mut testing::ScriptRunner<Step>,
    position: &str,
    raw: &str,
    reason: &str,
) -> testing::ScriptVerdict {
    tracing::error!(target: "script", "script failed step={position} {raw} reason={reason}");
    runner.finished_step_timings.push(testing::StepTiming {
        raw: raw.to_string(),
        secs: runner.step_elapsed.as_secs_f64(),
        failure: Some(reason.to_string()),
    });
    runner.verdict = testing::ScriptVerdict::Failed;
    runner.verdict
}

#[cfg(test)]
mod tests {
    use super::fail;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut runner = testing::ScriptRunner::<testing::E2EStep>::new(Vec::new());
        let verdict = fail(&mut runner, "1/1", "step-0", "boom");
        assert_eq!(verdict, testing::ScriptVerdict::Failed);
        assert_eq!(runner.verdict, testing::ScriptVerdict::Failed);
        assert_eq!(runner.finished_step_timings.len(), 1);
        assert_eq!(runner.finished_step_timings[0].raw, "step-0");
        assert_eq!(
            runner.finished_step_timings[0].failure.as_deref(),
            Some("boom")
        );
    }
}
