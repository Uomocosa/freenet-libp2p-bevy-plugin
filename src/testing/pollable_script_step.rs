use bevy::ecs::system::SystemParam;

use crate::testing;

pub trait PollableScriptStep: testing::ScriptStep {
    type Param<'w, 's>: SystemParam;

    fn poll(
        step: &Self,
        clock: testing::StepClock,
        param: &mut Self::Param<'_, '_>,
    ) -> testing::StepOutcome;
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::PollableScriptStep;
    use crate::testing;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DummyStep;

    impl testing::ScriptStep for DummyStep {
        const STEP_NAMES: &'static str = "dummy";

        fn parse_verb(_verb: &str, _arg: &str) -> Option<Result<Self, String>> {
            Some(Ok(Self))
        }

        fn is_wait(&self) -> bool {
            false
        }
    }

    impl PollableScriptStep for DummyStep {
        type Param<'w, 's> = ();

        fn poll(
            _step: &Self,
            _clock: testing::StepClock,
            _param: &mut Self::Param<'_, '_>,
        ) -> testing::StepOutcome {
            testing::StepOutcome::Done
        }
    }

    #[test]
    fn test_usage() {
        let clock = testing::StepClock {
            elapsed: Duration::ZERO,
            is_first_poll: true,
        };
        let mut unit = ();
        assert_eq!(
            <DummyStep as PollableScriptStep>::poll(&DummyStep, clock, &mut unit),
            testing::StepOutcome::Done
        );
    }
}
