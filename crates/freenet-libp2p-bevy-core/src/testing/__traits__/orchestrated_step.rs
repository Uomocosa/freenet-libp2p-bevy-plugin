use crate::testing;

pub trait OrchestratedStep: testing::traits::ScriptStep {
    fn script_text(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::OrchestratedStep;
    use crate::testing;
    use crate::testing::traits::ScriptStep;

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

    impl OrchestratedStep for ToyStep {
        fn script_text(&self) -> String {
            "toy".to_string()
        }
    }

    #[test]
    fn test_usage() {
        assert_eq!(ToyStep.script_text(), "toy");
        assert!(!ToyStep.is_wait());
    }
}
