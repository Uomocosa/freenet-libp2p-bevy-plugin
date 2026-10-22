use std::time::Duration;

/// The vocabulary of one example's `--script`: how a verb parses, and which steps wait.
pub trait ScriptStepGrammar: Sized {
    /// Every verb the grammar knows, for the "unknown step" message.
    const STEP_NAMES: &'static str;

    /// Parses `verb:arg`; `None` when the verb is not part of this grammar.
    fn parse_verb(verb: &str, arg: &str) -> Option<Result<Self, String>>;

    /// Whether the step waits for a condition, and so must carry an `@Ns` deadline.
    fn is_wait(&self) -> bool;
}

/// One step of a `--script`, with the deadline its wait must meet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimedScriptStep<Step> {
    pub step: Step,
    pub deadline: Option<Duration>,
    pub raw: String,
}

/// Parses one `verb[:arg][@Ns]` step. A wait must carry a deadline; any other step must not.
///
/// # Errors
///
/// Returns a message naming the offending part when the verb is unknown, an argument does
/// not parse, or the deadline is missing, misplaced or malformed.
pub fn parse_timed_script_step<Step: ScriptStepGrammar>(
    raw: &str,
) -> Result<TimedScriptStep<Step>, String> {
    let text = raw.trim();
    let (body, deadline) = match text.rsplit_once('@') {
        Some((body, deadline)) => (body, Some(parse_seconds(deadline)?)),
        None => (text, None),
    };
    let (verb, arg) = body.split_once(':').unwrap_or((body, ""));
    let step = Step::parse_verb(verb, arg).unwrap_or_else(|| {
        Err(format!(
            "unknown step '{verb}'; expected one of {}",
            Step::STEP_NAMES
        ))
    })?;
    match (step.is_wait(), deadline) {
        (true, None) => Err(format!(
            "step '{text}' is a wait and needs a deadline, e.g. '{text}@30s'"
        )),
        (false, Some(_)) => Err(format!(
            "step '{text}' is not a wait; only waits take '@Ns'"
        )),
        _ => Ok(TimedScriptStep {
            step,
            deadline,
            raw: text.to_string(),
        }),
    }
}

/// Parses a whole-seconds duration written as `Ns`, e.g. `30s`.
///
/// # Errors
///
/// Returns a message when the text is not a number followed by `s`.
pub fn parse_seconds(raw: &str) -> Result<Duration, String> {
    raw.trim()
        .strip_suffix('s')
        .and_then(|secs| secs.parse::<u64>().ok())
        .map(Duration::from_secs)
        .ok_or_else(|| format!("expected seconds like '30s', got '{raw}'"))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{ScriptStepGrammar, TimedScriptStep, parse_seconds, parse_timed_script_step};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ToyStep {
        Go,
        WaitReady,
    }

    impl ScriptStepGrammar for ToyStep {
        const STEP_NAMES: &'static str = "go, wait-ready";

        fn parse_verb(verb: &str, _arg: &str) -> Option<Result<Self, String>> {
            match verb {
                "go" => Some(Ok(Self::Go)),
                "wait-ready" => Some(Ok(Self::WaitReady)),
                _ => None,
            }
        }

        fn is_wait(&self) -> bool {
            matches!(self, Self::WaitReady)
        }
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            parse_timed_script_step::<ToyStep>(" wait-ready@30s "),
            Ok(TimedScriptStep {
                step: ToyStep::WaitReady,
                deadline: Some(Duration::from_secs(30)),
                raw: "wait-ready@30s".to_string(),
            })
        );
        assert_eq!(
            parse_timed_script_step::<ToyStep>("go").map(|timed| timed.step),
            Ok(ToyStep::Go)
        );
    }

    #[test]
    fn a_wait_without_a_deadline_is_rejected() {
        let error = parse_timed_script_step::<ToyStep>("wait-ready").expect_err("no deadline");
        assert!(error.contains("needs a deadline"));
    }

    #[test]
    fn an_action_with_a_deadline_is_rejected() {
        let error = parse_timed_script_step::<ToyStep>("go@5s").expect_err("misplaced");
        assert!(error.contains("not a wait"));
    }

    #[test]
    fn an_unknown_verb_names_the_known_ones() {
        let error = parse_timed_script_step::<ToyStep>("fly").expect_err("unknown");
        assert!(error.contains("go, wait-ready"));
    }

    #[test]
    fn seconds_need_the_s_suffix() {
        assert_eq!(parse_seconds("45s"), Ok(Duration::from_secs(45)));
        assert!(parse_seconds("45").is_err());
        assert!(parse_seconds("s").is_err());
    }
}
