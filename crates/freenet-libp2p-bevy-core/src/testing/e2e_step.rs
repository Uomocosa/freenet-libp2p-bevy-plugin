use atomic_delegate_macros::atomic_delegates;

use crate::identity;
use crate::testing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum E2EStep {
    CreateRoom(identity::RoomName),
    JoinRoom(identity::RoomName),
    LeaveRoom(identity::RoomName),
    WaitLobbyRoom(identity::RoomName),
    WaitLobbyRoomGone(identity::RoomName),
    WaitJoinSucceeded(identity::RoomName),
    WaitJoinFailed(identity::RoomName),
    WaitLeft(identity::RoomName),
    WaitMembers(testing::MemberCount),
    WaitMembersAreStrictlyLessThan(testing::MemberCount),
    WaitMembersAreLessOrEqualTo(testing::MemberCount),
    WaitRoomMessages(testing::SenderCount),
    WaitRingJoined,
    WaitLobbyReady,
    Sleep(std::time::Duration),
    MarkCompleted(testing::ScriptCheckpoint),
    ExitApp,
    CrashApp,
    Stay,
}

#[rustfmt::skip]
impl testing::traits::ScriptStep for E2EStep {
    const STEP_NAMES: &'static str = "create, join, leave, wait-lobby-room, wait-lobby-room-gone, \
        wait-join-succeeded, wait-join-failed, wait-left, wait-members, \
        wait-members-are-strictly-less-than, wait-members-are-less-or-equal-to, wait-room-messages, \
        wait-ring-joined, wait-lobby-ready, sleep, mark-completed, exit-app, crash-app, stay";
    fn parse_verb(verb: &str, arg: &str) -> Option<Result<Self, String>> {
        let room = || Self::room_name(verb, arg);
        Some(match verb {
            "create" => room().map(Self::CreateRoom),
            "join" => room().map(Self::JoinRoom),
            "leave" => room().map(Self::LeaveRoom),
            "wait-lobby-room" => room().map(Self::WaitLobbyRoom),
            "wait-lobby-room-gone" => room().map(Self::WaitLobbyRoomGone),
            "wait-join-succeeded" => room().map(Self::WaitJoinSucceeded),
            "wait-join-failed" => room().map(Self::WaitJoinFailed),
            "wait-left" => room().map(Self::WaitLeft),
            "wait-members" => Self::count(verb, arg).map(|n| Self::WaitMembers(testing::MemberCount(n))),
            "wait-members-are-strictly-less-than" => {
                Self::count(verb, arg).map(|n| Self::WaitMembersAreStrictlyLessThan(testing::MemberCount(n)))
            }
            "wait-members-are-less-or-equal-to" => {
                Self::count(verb, arg).map(|n| Self::WaitMembersAreLessOrEqualTo(testing::MemberCount(n)))
            }
            "wait-room-messages" => {
                Self::count(verb, arg).map(|n| Self::WaitRoomMessages(testing::SenderCount(n)))
            }
            "wait-ring-joined" => Self::unit(verb, arg).map(|()| Self::WaitRingJoined),
            "wait-lobby-ready" => Self::unit(verb, arg).map(|()| Self::WaitLobbyReady),
            "sleep" => testing::TimedScriptStep::<Self>::parse_seconds(arg).map(Self::Sleep),
            "mark-completed" => Self::checkpoint(verb, arg).map(Self::MarkCompleted),
            "exit-app" => Ok(Self::ExitApp),
            "crash-app" => Ok(Self::CrashApp),
            "stay" => Ok(Self::Stay),
            _ => return None,
        })
    }

    fn is_wait(&self) -> bool {
        matches!(
            self,
            Self::WaitLobbyRoom(_)
                | Self::WaitLobbyRoomGone(_)
                | Self::WaitJoinSucceeded(_)
                | Self::WaitJoinFailed(_)
                | Self::WaitLeft(_)
                | Self::WaitMembers(_)
                | Self::WaitMembersAreStrictlyLessThan(_)
                | Self::WaitMembersAreLessOrEqualTo(_)
                | Self::WaitRoomMessages(_)
                | Self::WaitRingJoined
                | Self::WaitLobbyReady
        )
    }
}

#[rustfmt::skip]
impl testing::traits::OrchestratedStep for E2EStep {
    fn script_text(&self) -> String {
        match self {
            Self::CreateRoom(name) => format!("create:{name}"),
            Self::JoinRoom(name) => format!("join:{name}"),
            Self::LeaveRoom(name) => format!("leave:{name}"),
            Self::WaitLobbyRoom(name) => format!("wait-lobby-room:{name}"),
            Self::WaitLobbyRoomGone(name) => format!("wait-lobby-room-gone:{name}"),
            Self::WaitJoinSucceeded(name) => format!("wait-join-succeeded:{name}"),
            Self::WaitJoinFailed(name) => format!("wait-join-failed:{name}"),
            Self::WaitLeft(name) => format!("wait-left:{name}"),
            Self::WaitMembers(count) => format!("wait-members:{}", **count),
            Self::WaitMembersAreStrictlyLessThan(count) => {
                format!("wait-members-are-strictly-less-than:{}", **count)
            }
            Self::WaitMembersAreLessOrEqualTo(count) => {
                format!("wait-members-are-less-or-equal-to:{}", **count)
            }
            Self::WaitRoomMessages(count) => format!("wait-room-messages:{}", **count),
            Self::WaitRingJoined => "wait-ring-joined".to_string(),
            Self::WaitLobbyReady => "wait-lobby-ready".to_string(),
            Self::Sleep(duration) => format!("sleep:{}s", duration.as_secs()),
            Self::MarkCompleted(checkpoint) => {
                format!("mark-completed:{checkpoint}")
            }
            Self::ExitApp => "exit-app".to_string(),
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
        view: &testing::E2EStepView<'_>,
    ) -> testing::E2EStepPoll {
    }
}

#[atomic_delegates]
impl E2EStep {
    pub(crate) fn join_failure_reason(
        view: &testing::E2EStepView<'_>,
        name: &identity::RoomName,
    ) -> Option<String> {
    }
}

#[rustfmt::skip]
impl E2EStep {
    pub(crate) fn unit(verb: &str, arg: &str) -> Result<(), String> {
        if arg.trim().is_empty() {
            Ok(())
        } else {
            Err(format!("step '{verb}' takes no argument, got '{arg}'"))
        }
    }

    pub(crate) fn room_name(verb: &str, arg: &str) -> Result<identity::RoomName, String> {
        let name = arg.trim();
        if name.is_empty() {
            return Err(format!("step '{verb}' needs a room name"));
        }
        Ok(identity::RoomName::from(name))
    }

    pub(crate) fn count(verb: &str, arg: &str) -> Result<u32, String> {
        arg.trim()
            .parse::<u32>()
            .map_err(|_| format!("step '{verb}' needs a count, got '{arg}'"))
    }

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
    use crate::identity;
    use crate::testing;

    fn step(raw: &str) -> Result<E2EStep, String> {
        testing::TimedScriptStep::<E2EStep>::try_from(raw).map(|timed| timed.step)
    }

    #[test]
    fn test_usage() {
        let room = identity::RoomName::from("room-1");
        assert_eq!(step("create:room-1"), Ok(E2EStep::CreateRoom(room.clone())));
        assert_eq!(
            step("wait-join-succeeded:room-1@30s"),
            Ok(E2EStep::WaitJoinSucceeded(room))
        );
        assert_eq!(
            step("wait-members:3@60s"),
            Ok(E2EStep::WaitMembers(testing::MemberCount(3)))
        );
        assert_eq!(
            step("sleep:45s"),
            Ok(E2EStep::Sleep(Duration::from_secs(45)))
        );
        assert_eq!(step("exit-app"), Ok(E2EStep::ExitApp));
        assert_eq!(step("crash-app"), Ok(E2EStep::CrashApp));
        assert_eq!(step("stay"), Ok(E2EStep::Stay));
    }

    #[test]
    fn a_room_step_needs_a_room_name() {
        assert!(step("join").is_err());
        assert!(step("wait-left:@10s").is_err());
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

    #[test]
    fn ring_joined_waits_for_the_first_ring_peer() {
        assert_eq!(step("wait-ring-joined@120s"), Ok(E2EStep::WaitRingJoined));
        assert!(step("wait-ring-joined").is_err());
        assert!(step("wait-ring-joined:1@120s").is_err());
    }

    #[test]
    fn lobby_ready_waits_for_the_lobby_fetch() {
        assert_eq!(step("wait-lobby-ready@60s"), Ok(E2EStep::WaitLobbyReady));
        assert!(step("wait-lobby-ready").is_err());
        assert!(step("wait-lobby-ready:room-1@60s").is_err());
    }
}
