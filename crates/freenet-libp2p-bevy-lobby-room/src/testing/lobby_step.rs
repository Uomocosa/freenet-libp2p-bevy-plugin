use atomic_delegate_macros::{atomic_delegate, atomic_delegates};

use crate::identity;
use crate::testing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LobbyStep {
    CreateRoom(identity::RoomName),
    JoinRoom(identity::RoomName),
    LeaveRoom(identity::RoomName),
    WaitLobbyRoom(identity::RoomName),
    WaitLobbyRoomGone(identity::RoomName),
    WaitJoinSucceeded(identity::RoomName),
    WaitJoinFailed(identity::RoomName),
    WaitLeft(identity::RoomName),
    WaitRingJoined,
    WaitLobbyReady,
    ExitApp,
}

impl testing::traits::ScriptStep for LobbyStep {
    const STEP_NAMES: &'static str = "create, join, leave, wait-lobby-room, wait-lobby-room-gone, \
        wait-join-succeeded, wait-join-failed, wait-left, wait-ring-joined, wait-lobby-ready, \
        exit-app";

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
            "wait-ring-joined" => Self::unit(verb, arg).map(|()| Self::WaitRingJoined),
            "wait-lobby-ready" => Self::unit(verb, arg).map(|()| Self::WaitLobbyReady),
            "exit-app" => Ok(Self::ExitApp),
            _ => return None,
        })
    }

    #[atomic_delegate(LobbyStep, ScriptStep)]
    fn is_wait(&self) -> bool {}
}

impl testing::traits::OrchestratedStep for LobbyStep {
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
            Self::WaitRingJoined => "wait-ring-joined".to_string(),
            Self::WaitLobbyReady => "wait-lobby-ready".to_string(),
            Self::ExitApp => "exit-app".to_string(),
        }
    }
}

#[atomic_delegates]
impl LobbyStep {
    #[must_use]
    pub fn poll_lobby_step(
        &self,
        clock: testing::StepClock,
        view: &testing::LobbyStepView<'_>,
    ) -> testing::E2EStepPoll {
    }
}

#[atomic_delegates]
impl LobbyStep {
    pub(crate) fn join_failure_reason(
        view: &testing::LobbyStepView<'_>,
        name: &identity::RoomName,
    ) -> Option<String> {
    }
}

impl LobbyStep {
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
}

#[cfg(test)]
mod tests {
    use super::LobbyStep;
    use crate::identity;
    use crate::testing;

    fn step(raw: &str) -> Result<LobbyStep, String> {
        testing::TimedScriptStep::<LobbyStep>::try_from(raw).map(|timed| timed.step)
    }

    #[test]
    fn test_usage() {
        let room = identity::RoomName::from("room-1");
        assert_eq!(
            step("create:room-1"),
            Ok(LobbyStep::CreateRoom(room.clone()))
        );
        assert_eq!(
            step("wait-join-succeeded:room-1@30s"),
            Ok(LobbyStep::WaitJoinSucceeded(room))
        );
        assert_eq!(step("exit-app"), Ok(LobbyStep::ExitApp));
    }

    #[test]
    fn a_room_step_needs_a_room_name() {
        assert!(step("join").is_err());
        assert!(step("wait-left:@10s").is_err());
    }

    #[test]
    fn ring_joined_waits_for_the_first_ring_peer() {
        assert_eq!(step("wait-ring-joined@120s"), Ok(LobbyStep::WaitRingJoined));
        assert!(step("wait-ring-joined").is_err());
        assert!(step("wait-ring-joined:1@120s").is_err());
    }

    #[test]
    fn lobby_ready_waits_for_the_lobby_fetch() {
        assert_eq!(step("wait-lobby-ready@60s"), Ok(LobbyStep::WaitLobbyReady));
        assert!(step("wait-lobby-ready").is_err());
        assert!(step("wait-lobby-ready:room-1@60s").is_err());
    }
}
