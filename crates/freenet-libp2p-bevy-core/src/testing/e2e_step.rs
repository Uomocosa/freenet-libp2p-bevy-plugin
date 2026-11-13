use atomic_delegate_macros::{atomic_delegate, atomic_delegates};

use crate::testing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum E2EStep {
    WaitRoomMessages(testing::SenderCount),
    WaitMembers(testing::MemberCount),
    WaitMembersAreStrictlyLessThan(testing::MemberCount),
    WaitMembersAreLessOrEqualTo(testing::MemberCount),
    Sleep(std::time::Duration),
    MarkCompleted(testing::ScriptCheckpoint),
    CrashApp,
    Stay,
}

impl testing::traits::ScriptStep for E2EStep {
    const STEP_NAMES: &'static str = "wait-members, \
        wait-members-are-strictly-less-than, wait-members-are-less-or-equal-to, wait-room-messages, \
        sleep, mark-completed, crash-app, stay";
    fn parse_verb(verb: &str, arg: &str) -> Option<Result<Self, String>> {
        Some(match verb {
            "wait-members" => {
                Self::count(verb, arg).map(|n| Self::WaitMembers(testing::MemberCount(n)))
            }
            "wait-members-are-strictly-less-than" => Self::count(verb, arg)
                .map(|n| Self::WaitMembersAreStrictlyLessThan(testing::MemberCount(n))),
            "wait-members-are-less-or-equal-to" => Self::count(verb, arg)
                .map(|n| Self::WaitMembersAreLessOrEqualTo(testing::MemberCount(n))),
            "wait-room-messages" => {
                Self::count(verb, arg).map(|n| Self::WaitRoomMessages(testing::SenderCount(n)))
            }
            "sleep" => testing::TimedScriptStep::<Self>::parse_seconds(arg).map(Self::Sleep),
            "mark-completed" => Self::checkpoint(verb, arg).map(Self::MarkCompleted),
            "crash-app" => Ok(Self::CrashApp),
            "stay" => Ok(Self::Stay),
            _ => return None,
        })
    }

    #[atomic_delegate(E2EStep, ScriptStep)]
    fn is_wait(&self) -> bool {}
}

impl testing::traits::OrchestratedStep for E2EStep {
    fn script_text(&self) -> String {
        match self {
            Self::WaitMembers(count) => format!("wait-members:{}", **count),
            Self::WaitMembersAreStrictlyLessThan(count) => {
                format!("wait-members-are-strictly-less-than:{}", **count)
            }
            Self::WaitMembersAreLessOrEqualTo(count) => {
                format!("wait-members-are-less-or-equal-to:{}", **count)
            }
            Self::WaitRoomMessages(count) => format!("wait-room-messages:{}", **count),
            Self::Sleep(duration) => format!("sleep:{}s", duration.as_secs()),
            Self::MarkCompleted(checkpoint) => {
                format!("mark-completed:{checkpoint}")
            }
            Self::CrashApp => "crash-app".to_string(),
            Self::Stay => "stay".to_string(),
        }
    }
}

#[atomic_delegates]
impl E2EStep {
    #[must_use]
    pub fn poll_shared_script_step(
        &self,
        clock: testing::StepClock,
        view: &testing::E2EStepView,
    ) -> testing::E2EStepPoll {
    }
}

#[atomic_delegates]
impl E2EStep {
    pub(crate) fn count(verb: &str, arg: &str) -> Result<u32, String> {}
}

impl E2EStep {
    pub(crate) fn checkpoint(verb: &str, arg: &str) -> Result<testing::ScriptCheckpoint, String> {
        let name = arg.trim();
        if name.is_empty() {
            return Err(format!("step '{verb}' needs a checkpoint name"));
        }
        Ok(testing::ScriptCheckpoint::new(Box::leak(
            name.to_owned().into_boxed_str(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::E2EStep;
    use crate::testing;

    fn step(raw: &str) -> Result<E2EStep, String> {
        testing::TimedScriptStep::<E2EStep>::try_from(raw).map(|timed| timed.step)
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            step("wait-members:3@60s"),
            Ok(E2EStep::WaitMembers(testing::MemberCount(3)))
        );
        assert_eq!(
            step("sleep:45s"),
            Ok(E2EStep::Sleep(Duration::from_secs(45)))
        );
        assert_eq!(step("crash-app"), Ok(E2EStep::CrashApp));
        assert_eq!(step("stay"), Ok(E2EStep::Stay));
    }

    #[test]
    fn sleep_is_not_a_wait() {
        assert!(step("sleep:5s@10s").is_err());
    }

    #[test]
    fn bounded_member_waits_parse_like_wait_members() {
        assert_eq!(
            step("wait-members-are-strictly-less-than:2@60s"),
            Ok(E2EStep::WaitMembersAreStrictlyLessThan(
                testing::MemberCount(2)
            ))
        );
        assert_eq!(
            step("wait-members-are-less-or-equal-to:1@60s"),
            Ok(E2EStep::WaitMembersAreLessOrEqualTo(testing::MemberCount(
                1
            )))
        );
        assert!(step("wait-members-are-strictly-less-than:2").is_err());
        assert!(step("wait-members-are-less-or-equal-to:1").is_err());
        assert!(step("wait-members-are-strictly-less-than:x@60s").is_err());
    }
}
