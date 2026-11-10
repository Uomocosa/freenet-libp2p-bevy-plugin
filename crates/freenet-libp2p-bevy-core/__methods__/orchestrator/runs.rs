use crate::testing;

pub fn runs<Step>(
    orchestrator: &mut testing::Orchestrator,
    steps: impl IntoIterator<Item = Vec<testing::StepWithDeadline<Step>>>,
) -> &mut testing::Orchestrator
where
    Step: testing::traits::OrchestratedStep,
{
    let Some(last) = orchestrator.peers.last_mut() else {
        return orchestrator;
    };
    for group in steps {
        for pending in group {
            last.steps.push(testing::StoredStep {
                text: pending.step.script_text(),
                is_wait: pending.step.is_wait(),
                within: pending.within,
            });
        }
    }
    orchestrator
}

#[cfg(test)]
mod tests {
    use super::runs;
    use crate::testing;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ToyStep;

    impl testing::traits::ScriptStep for ToyStep {
        const STEP_NAMES: &'static str = "toy";

        fn parse_verb(verb: &str, _arg: &str) -> Option<Result<Self, String>> {
            match verb {
                "toy" => Some(Ok(Self)),
                _ => None,
            }
        }

        fn is_wait(&self) -> bool {
            false
        }
    }

    impl testing::traits::OrchestratedStep for ToyStep {
        fn script_text(&self) -> String {
            "toy".to_string()
        }
    }

    #[test]
    fn test_usage() {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::Fake,
        );
        orchestrator.peer("painter");
        let _ = runs(
            &mut orchestrator,
            [vec![testing::StepWithDeadline::from(ToyStep)]],
        );
        let first = orchestrator.peers.first().expect("peer");
        assert_eq!(first.steps.len(), 1);
        assert_eq!(
            first.steps.first().map(|step| step.text.as_str()),
            Some("toy")
        );
    }
}
