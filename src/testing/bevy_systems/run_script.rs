use bevy::prelude::*;

use crate::testing;

pub fn run_script<Step: testing::PollableScriptStep>(
    time: Res<Time>,
    mut runner: ResMut<testing::ScriptRunner<Step>>,
    mut exits: MessageWriter<AppExit>,
    mut param: Step::Param<'_, '_>,
) {
    let delta = time.into_inner().delta();
    let verdict = runner.advance(delta, |step, clock| Step::poll(step, clock, &mut param));
    if let Some(exit) = verdict.app_exit_for_verdict() {
        exits.write(exit);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;

    use super::run_script;
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

    impl testing::PollableScriptStep for DummyStep {
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
        let steps = std::iter::once(&"dummy")
            .map(|raw| testing::TimedScriptStep::<DummyStep>::try_from(*raw).expect("step"))
            .collect();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<AppExit>();
        app.insert_resource(testing::ScriptRunner::<DummyStep>::new(steps));
        app.add_systems(Update, run_script::<DummyStep>);
        app.update();
        assert_eq!(
            app.world()
                .resource::<testing::ScriptRunner<DummyStep>>()
                .index,
            1
        );
        assert_eq!(app.world().resource::<Messages<AppExit>>().len(), 1);
    }

    #[test]
    fn a_pending_step_keeps_running_without_an_exit() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct WaitingStep;

        impl testing::ScriptStep for WaitingStep {
            const STEP_NAMES: &'static str = "waiting";

            fn parse_verb(_verb: &str, _arg: &str) -> Option<Result<Self, String>> {
                Some(Ok(Self))
            }

            fn is_wait(&self) -> bool {
                true
            }
        }

        impl testing::PollableScriptStep for WaitingStep {
            type Param<'w, 's> = ();

            fn poll(
                _step: &Self,
                _clock: testing::StepClock,
                _param: &mut Self::Param<'_, '_>,
            ) -> testing::StepOutcome {
                testing::StepOutcome::Pending
            }
        }

        let timed = testing::TimedScriptStep {
            step: WaitingStep,
            deadline: Some(Duration::from_secs(60)),
            raw: "waiting@60s".to_string(),
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<AppExit>();
        app.insert_resource(testing::ScriptRunner::new(vec![timed]));
        app.add_systems(Update, run_script::<WaitingStep>);
        app.update();
        assert_eq!(
            app.world()
                .resource::<testing::ScriptRunner<WaitingStep>>()
                .index,
            0
        );
        assert!(app.world().resource::<Messages<AppExit>>().is_empty());
    }
}
