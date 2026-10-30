use std::time::Duration;

use freenet_libp2p_bevy_plugin::{discovery, identity};

use crate::shared::action::Action;
use crate::shared::script::script_runner::{StepClock, StepOutcome};
use crate::shared::script::shared_script_step::SharedScriptStep;

/// How long `exit-app` keeps running after it published its quit, so the tombstone lands.
pub const QUIT_PUBLISH_GRACE: Duration = Duration::from_secs(2);

/// What a shared step can see of the running app.
pub struct SharedStepView<'a> {
    pub status: &'a discovery::Status,
    pub join_error: Option<&'a str>,
    pub room_message_senders: usize,
}

/// The side effect a shared step asks the app to perform this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharedStepEffect {
    ArmRoomAction(Action),
    SendDiscoveryCommand(discovery::Command),
    AbortProcess,
}

/// A shared step's outcome this frame, plus the effect to perform, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedStepPoll {
    pub outcome: StepOutcome,
    pub effect: Option<SharedStepEffect>,
}

/// Decides, without touching the app, how far one shared step got this frame.
#[must_use]
pub fn poll_shared_script_step(
    step: &SharedScriptStep,
    clock: StepClock,
    view: &SharedStepView<'_>,
) -> SharedStepPoll {
    let room = view.status.room.as_ref();
    let is_in = |name: &identity::RoomName| room.is_some_and(|joined| joined.name == *name);
    let is_ready_in = |name: &identity::RoomName| {
        room.is_some_and(|joined| {
            joined.name == *name && joined.phase == discovery::JoinPhase::Ready
        })
    };
    let done_when = |is_done: bool| SharedStepPoll {
        outcome: if is_done {
            StepOutcome::Done
        } else {
            StepOutcome::Pending
        },
        effect: None,
    };
    match step {
        SharedScriptStep::CreateRoom(name) => SharedStepPoll {
            outcome: StepOutcome::Done,
            effect: Some(SharedStepEffect::ArmRoomAction(Action::Create(
                name.clone(),
            ))),
        },
        SharedScriptStep::JoinRoom(name) => SharedStepPoll {
            outcome: StepOutcome::Done,
            effect: Some(SharedStepEffect::ArmRoomAction(Action::Join(name.clone()))),
        },
        SharedScriptStep::LeaveRoom(_) => SharedStepPoll {
            outcome: StepOutcome::Done,
            effect: Some(SharedStepEffect::SendDiscoveryCommand(
                discovery::Command::Leave,
            )),
        },
        SharedScriptStep::WaitLobbyRoom(name) => done_when(view.status.lobby.contains_key(name)),
        SharedScriptStep::WaitJoinSucceeded(name) => view.join_error.map_or_else(
            || done_when(is_ready_in(name)),
            |reason| SharedStepPoll {
                outcome: StepOutcome::Failed(format!("join failed: {reason}")),
                effect: None,
            },
        ),
        // discovery's Ready only means the room is listed; a joiner still waiting for history
        // can fail later, so a wrong success shows up as the step's deadline
        SharedScriptStep::WaitJoinFailed(name) => {
            done_when(join_failure_reason(view, name).is_some())
        }
        SharedScriptStep::WaitLeft(name) => done_when(!is_in(name)),
        SharedScriptStep::WaitMembers(count) => {
            let members = view.status.connected_members().count().saturating_add(1);
            done_when(u32::try_from(members).unwrap_or(u32::MAX) >= **count)
        }
        SharedScriptStep::WaitMembersAreStrictlyLessThan(count) => {
            let members = view.status.connected_members().count().saturating_add(1);
            done_when(u32::try_from(members).unwrap_or(u32::MAX) < **count)
        }
        SharedScriptStep::WaitMembersAreLessOrEqualTo(count) => {
            let members = view.status.connected_members().count().saturating_add(1);
            done_when(u32::try_from(members).unwrap_or(u32::MAX) <= **count)
        }
        SharedScriptStep::WaitRoomMessages(count) => {
            done_when(u32::try_from(view.room_message_senders).unwrap_or(u32::MAX) >= **count)
        }
        SharedScriptStep::Sleep(duration) => done_when(clock.elapsed >= *duration),
        SharedScriptStep::ExitApp => SharedStepPoll {
            outcome: if clock.elapsed >= QUIT_PUBLISH_GRACE {
                StepOutcome::Done
            } else {
                StepOutcome::Pending
            },
            effect: clock
                .is_first_poll
                .then_some(SharedStepEffect::SendDiscoveryCommand(
                    discovery::Command::Quit,
                )),
        },
        SharedScriptStep::CrashApp => SharedStepPoll {
            outcome: StepOutcome::Pending,
            effect: Some(SharedStepEffect::AbortProcess),
        },
        SharedScriptStep::Stay => done_when(false),
    }
}

// needed helper: why the join to `name` failed, if it did — the plugin's error, or the room
// vanishing from an attached lobby while the peer is not in it
fn join_failure_reason(view: &SharedStepView<'_>, name: &identity::RoomName) -> Option<String> {
    if let Some(reason) = view.join_error {
        return Some(reason.to_string());
    }
    let status = view.status;
    if status.room.is_some() || !status.is_lobby_attached || status.lobby.contains_key(name) {
        return None;
    }
    Some(format!("room {name} no longer exists"))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::{SharedStepEffect, SharedStepView, poll_shared_script_step};
    use crate::shared::script::script_runner::{StepClock, StepOutcome};
    use crate::shared::script::shared_script_step::SharedScriptStep;

    const FIRST: StepClock = StepClock {
        elapsed: Duration::ZERO,
        is_first_poll: true,
    };

    fn view(status: &discovery::Status) -> SharedStepView<'_> {
        SharedStepView {
            status,
            join_error: None,
            room_message_senders: 0,
        }
    }

    #[test]
    fn test_usage() {
        let status = discovery::Status::default();
        let room = identity::RoomName::from("room-1");
        let poll = poll_shared_script_step(
            &SharedScriptStep::JoinRoom(room.clone()),
            FIRST,
            &view(&status),
        );
        assert_eq!(poll.outcome, StepOutcome::Done);
        assert!(matches!(
            poll.effect,
            Some(SharedStepEffect::ArmRoomAction(_))
        ));
        let poll = poll_shared_script_step(
            &SharedScriptStep::WaitJoinSucceeded(room),
            FIRST,
            &view(&status),
        );
        assert_eq!(poll.outcome, StepOutcome::Pending);
    }

    #[test]
    fn a_join_error_fails_the_join_wait_at_once() {
        let status = discovery::Status::default();
        let view = SharedStepView {
            join_error: Some("room gone"),
            ..view(&status)
        };
        let poll = poll_shared_script_step(
            &SharedScriptStep::WaitJoinSucceeded(identity::RoomName::from("room-1")),
            FIRST,
            &view,
        );
        assert!(matches!(poll.outcome, StepOutcome::Failed(_)));
    }

    #[test]
    fn a_room_missing_from_the_attached_lobby_is_a_failed_join() {
        let status = discovery::Status {
            is_lobby_attached: true,
            ..discovery::Status::default()
        };
        let poll = poll_shared_script_step(
            &SharedScriptStep::WaitJoinFailed(identity::RoomName::from("room-1")),
            FIRST,
            &view(&status),
        );
        assert_eq!(poll.outcome, StepOutcome::Done);
    }

    #[test]
    fn exit_app_quits_once_then_waits_the_grace() {
        let status = discovery::Status::default();
        let first = poll_shared_script_step(&SharedScriptStep::ExitApp, FIRST, &view(&status));
        assert_eq!(
            first.effect,
            Some(SharedStepEffect::SendDiscoveryCommand(
                discovery::Command::Quit
            ))
        );
        assert_eq!(first.outcome, StepOutcome::Pending);
        let later = StepClock {
            elapsed: Duration::from_secs(3),
            is_first_poll: false,
        };
        let last = poll_shared_script_step(&SharedScriptStep::ExitApp, later, &view(&status));
        assert_eq!(last.effect, None);
        assert_eq!(last.outcome, StepOutcome::Done);
    }

    #[test]
    fn strictly_less_member_wait_completes_below_the_bound() {
        use crate::shared::script::shared_script_step::MemberCount;
        let status = discovery::Status::default();
        let alone = view(&status);
        let done = poll_shared_script_step(
            &SharedScriptStep::WaitMembersAreStrictlyLessThan(MemberCount(2)),
            FIRST,
            &alone,
        );
        assert_eq!(done.outcome, StepOutcome::Done);
        let pending = poll_shared_script_step(
            &SharedScriptStep::WaitMembersAreStrictlyLessThan(MemberCount(1)),
            FIRST,
            &alone,
        );
        assert_eq!(pending.outcome, StepOutcome::Pending);
    }

    #[test]
    fn less_or_equal_member_wait_completes_at_the_bound() {
        use crate::shared::script::shared_script_step::MemberCount;
        let status = discovery::Status::default();
        let alone = view(&status);
        let done = poll_shared_script_step(
            &SharedScriptStep::WaitMembersAreLessOrEqualTo(MemberCount(1)),
            FIRST,
            &alone,
        );
        assert_eq!(done.outcome, StepOutcome::Done);
        let pending = poll_shared_script_step(
            &SharedScriptStep::WaitMembersAreLessOrEqualTo(MemberCount(0)),
            FIRST,
            &alone,
        );
        assert_eq!(pending.outcome, StepOutcome::Pending);
    }
}
