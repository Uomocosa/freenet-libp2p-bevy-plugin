use std::time::Duration;

use crate::testing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimedScriptStep<Step> {
    pub step: Step,
    pub deadline: Option<Duration>,
    pub raw: String,
}

impl<Step: testing::ScriptStep> TryFrom<&str> for TimedScriptStep<Step> {
    type Error = String;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        let text = raw.trim();
        let (verb, arg, deadline) = Self::split_head(text)?;
        Self::try_from_parts(verb, arg, deadline, text)
    }
}

impl<Step: testing::ScriptStep> TimedScriptStep<Step> {
    pub(crate) fn try_from_parts(
        verb: &str,
        arg: &str,
        deadline: Option<Duration>,
        text: &str,
    ) -> Result<Self, String> {
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
            _ => Ok(Self {
                step,
                deadline,
                raw: text.to_string(),
            }),
        }
    }
}

#[rustfmt::skip]
impl<Step: testing::ScriptStep> TimedScriptStep<Step> {
    pub(crate) fn split_head(text: &str) -> Result<(&str, &str, Option<Duration>), String> {
        let (body, deadline) = Self::split_deadline(text)?;
        let (verb, arg) = Self::split_verb(body);
        Ok((verb, arg, deadline))
    }

    pub(crate) fn split_deadline(text: &str) -> Result<(&str, Option<Duration>), String> {
        match text.rsplit_once('@') {
            Some((body, deadline)) => Ok((body, Some(Self::parse_seconds(deadline)?))),
            None => Ok((text, None)),
        }
    }

    pub(crate) fn split_verb(body: &str) -> (&str, &str) {
        body.split_once(':').unwrap_or((body, ""))
    }

    pub(crate) fn parse_seconds(raw: &str) -> Result<Duration, String> {
        raw.trim()
            .strip_suffix('s')
            .and_then(|secs| secs.parse::<u64>().ok())
            .map(Duration::from_secs)
            .ok_or_else(|| format!("expected seconds like '30s', got '{raw}'"))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::TimedScriptStep;
    use crate::testing;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ToyStep {
        Go,
        WaitReady,
    }

    impl testing::ScriptStep for ToyStep {
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
            TimedScriptStep::<ToyStep>::try_from(" wait-ready@30s "),
            Ok(TimedScriptStep {
                step: ToyStep::WaitReady,
                deadline: Some(Duration::from_secs(30)),
                raw: "wait-ready@30s".to_string(),
            })
        );
        assert_eq!(
            TimedScriptStep::<ToyStep>::try_from("go").map(|timed| timed.step),
            Ok(ToyStep::Go)
        );
    }

    #[test]
    fn a_wait_without_a_deadline_is_rejected() {
        let error = TimedScriptStep::<ToyStep>::try_from("wait-ready").expect_err("no deadline");
        assert!(error.contains("needs a deadline"));
    }

    #[test]
    fn an_action_with_a_deadline_is_rejected() {
        let error = TimedScriptStep::<ToyStep>::try_from("go@5s").expect_err("misplaced");
        assert!(error.contains("not a wait"));
    }

    #[test]
    fn an_unknown_verb_names_the_known_ones() {
        let error = TimedScriptStep::<ToyStep>::try_from("fly").expect_err("unknown");
        assert!(error.contains("go, wait-ready"));
    }

    #[test]
    fn seconds_need_the_s_suffix() {
        assert_eq!(
            TimedScriptStep::<ToyStep>::parse_seconds("45s"),
            Ok(Duration::from_secs(45))
        );
        assert!(TimedScriptStep::<ToyStep>::parse_seconds("45").is_err());
        assert!(TimedScriptStep::<ToyStep>::parse_seconds("s").is_err());
    }
}
