use std::time::Duration;

use derive_more::{Deref, From};
use freenet_libp2p_bevy_plugin::identity;

use crate::shared::script::timed_script_step::{ScriptStepGrammar, parse_seconds};

/// How many room members a step waits for, the peer itself included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, From)]
pub struct MemberCount(pub u32);

/// How many distinct peers a step waits to receive room messages from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, From)]
pub struct SenderCount(pub u32);

/// One step every scripted example understands: room actions, waits, and how the run ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharedScriptStep {
    CreateRoom(identity::RoomName),
    JoinRoom(identity::RoomName),
    LeaveRoom(identity::RoomName),
    WaitLobbyRoom(identity::RoomName),
    WaitJoinSucceeded(identity::RoomName),
    WaitJoinFailed(identity::RoomName),
    WaitLeft(identity::RoomName),
    WaitMembers(MemberCount),
    WaitMembersAreStrictlyLessThan(MemberCount),
    WaitMembersAreLessOrEqualTo(MemberCount),
    WaitRoomMessages(SenderCount),
    Sleep(Duration),
    ExitApp,
    CrashApp,
    Stay,
}

impl ScriptStepGrammar for SharedScriptStep {
    const STEP_NAMES: &'static str = "create, join, leave, wait-lobby-room, \
        wait-join-succeeded, wait-join-failed, wait-left, wait-members, \
        wait-members-are-strictly-less-than, wait-members-are-less-or-equal-to, wait-room-messages, \
        sleep, exit-app, crash-app, stay";

    fn parse_verb(verb: &str, arg: &str) -> Option<Result<Self, String>> {
        let room = || room_name(verb, arg);
        Some(match verb {
            "create" => room().map(Self::CreateRoom),
            "join" => room().map(Self::JoinRoom),
            "leave" => room().map(Self::LeaveRoom),
            "wait-lobby-room" => room().map(Self::WaitLobbyRoom),
            "wait-join-succeeded" => room().map(Self::WaitJoinSucceeded),
            "wait-join-failed" => room().map(Self::WaitJoinFailed),
            "wait-left" => room().map(Self::WaitLeft),
            "wait-members" => count(verb, arg).map(|n| Self::WaitMembers(MemberCount(n))),
            "wait-members-are-strictly-less-than" => {
                count(verb, arg).map(|n| Self::WaitMembersAreStrictlyLessThan(MemberCount(n)))
            }
            "wait-members-are-less-or-equal-to" => {
                count(verb, arg).map(|n| Self::WaitMembersAreLessOrEqualTo(MemberCount(n)))
            }
            "wait-room-messages" => {
                count(verb, arg).map(|n| Self::WaitRoomMessages(SenderCount(n)))
            }
            "sleep" => parse_seconds(arg).map(Self::Sleep),
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
                | Self::WaitJoinSucceeded(_)
                | Self::WaitJoinFailed(_)
                | Self::WaitLeft(_)
                | Self::WaitMembers(_)
                | Self::WaitMembersAreStrictlyLessThan(_)
                | Self::WaitMembersAreLessOrEqualTo(_)
                | Self::WaitRoomMessages(_)
        )
    }
}

// needed helper: a step's room name argument, which must not be empty
fn room_name(verb: &str, arg: &str) -> Result<identity::RoomName, String> {
    let name = arg.trim();
    if name.is_empty() {
        return Err(format!("step '{verb}' needs a room name"));
    }
    Ok(identity::RoomName::from(name))
}

// needed helper: a step's numeric argument, which must be a count
fn count(verb: &str, arg: &str) -> Result<u32, String> {
    arg.trim()
        .parse::<u32>()
        .map_err(|_| format!("step '{verb}' needs a count, got '{arg}'"))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use freenet_libp2p_bevy_plugin::identity;

    use super::{MemberCount, SharedScriptStep};
    use crate::shared::script::timed_script_step::parse_timed_script_step;

    fn step(raw: &str) -> Result<SharedScriptStep, String> {
        parse_timed_script_step::<SharedScriptStep>(raw).map(|timed| timed.step)
    }

    #[test]
    fn test_usage() {
        let room = identity::RoomName::from("room-1");
        assert_eq!(
            step("create:room-1"),
            Ok(SharedScriptStep::CreateRoom(room.clone()))
        );
        assert_eq!(
            step("wait-join-succeeded:room-1@30s"),
            Ok(SharedScriptStep::WaitJoinSucceeded(room))
        );
        assert_eq!(
            step("wait-members:3@60s"),
            Ok(SharedScriptStep::WaitMembers(MemberCount(3)))
        );
        assert_eq!(
            step("sleep:45s"),
            Ok(SharedScriptStep::Sleep(Duration::from_secs(45)))
        );
        assert_eq!(step("exit-app"), Ok(SharedScriptStep::ExitApp));
        assert_eq!(step("crash-app"), Ok(SharedScriptStep::CrashApp));
        assert_eq!(step("stay"), Ok(SharedScriptStep::Stay));
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
            Ok(SharedScriptStep::WaitMembersAreStrictlyLessThan(
                MemberCount(2)
            ))
        );
        assert_eq!(
            step("wait-members-are-less-or-equal-to:1@60s"),
            Ok(SharedScriptStep::WaitMembersAreLessOrEqualTo(MemberCount(
                1
            )))
        );
        assert!(step("wait-members-are-strictly-less-than:2").is_err());
        assert!(step("wait-members-are-less-or-equal-to:1").is_err());
        assert!(step("wait-members-are-strictly-less-than:x@60s").is_err());
    }
}
