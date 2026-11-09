pub trait ScriptStep: Clone + Send + Sync + 'static {
    const STEP_NAMES: &'static str;

    fn parse_verb(verb: &str, arg: &str) -> Option<Result<Self, String>>;

    fn is_wait(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::ScriptStep;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ToyStep {
        Go,
        Wait,
    }

    impl ScriptStep for ToyStep {
        const STEP_NAMES: &'static str = "go, wait";

        fn parse_verb(verb: &str, _arg: &str) -> Option<Result<Self, String>> {
            match verb {
                "go" => Some(Ok(Self::Go)),
                "wait" => Some(Ok(Self::Wait)),
                _ => None,
            }
        }

        fn is_wait(&self) -> bool {
            matches!(self, Self::Wait)
        }
    }

    #[test]
    fn test_usage() {
        assert_eq!(ToyStep::parse_verb("go", ""), Some(Ok(ToyStep::Go)));
        assert_eq!(ToyStep::parse_verb("wait", ""), Some(Ok(ToyStep::Wait)));
        assert_eq!(ToyStep::parse_verb("fly", ""), None);
        assert!(ToyStep::Wait.is_wait());
        assert!(!ToyStep::Go.is_wait());
    }
}
