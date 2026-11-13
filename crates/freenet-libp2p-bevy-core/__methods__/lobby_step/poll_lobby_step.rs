use crate::discovery;
use crate::identity;
use crate::testing;

#[must_use]
pub fn poll_lobby_step(
    step: &testing::LobbyStep,
    clock: testing::StepClock,
    view: &testing::LobbyStepView<'_>,
) -> testing::E2EStepPoll {
    let room = view.status.room.as_ref();
    let is_in = |name: &identity::RoomName| room.is_some_and(|joined| joined.name == *name);
    let is_ready_in = |name: &identity::RoomName| {
        room.is_some_and(|joined| {
            joined.name == *name && joined.phase == discovery::JoinPhase::Ready
        })
    };
    let done_when = |is_done: bool| testing::E2EStepPoll {
        outcome: if is_done {
            testing::StepOutcome::Done
        } else {
            testing::StepOutcome::Pending
        },
        effect: None,
    };
    match step {
        testing::LobbyStep::CreateRoom(name) => testing::E2EStepPoll {
            outcome: testing::StepOutcome::Done,
            effect: Some(testing::E2EStepEffect::ArmRoomAction(
                testing::Action::Create(name.clone()),
            )),
        },
        testing::LobbyStep::JoinRoom(name) => testing::E2EStepPoll {
            outcome: testing::StepOutcome::Done,
            effect: Some(testing::E2EStepEffect::ArmRoomAction(
                testing::Action::Join(name.clone()),
            )),
        },
        testing::LobbyStep::LeaveRoom(_) => testing::E2EStepPoll {
            outcome: testing::StepOutcome::Done,
            effect: Some(testing::E2EStepEffect::Extension(
                discovery::Command::Leave.to_extension_bytes(),
            )),
        },
        testing::LobbyStep::WaitLobbyRoom(name) => done_when(view.status.lobby.contains_key(name)),
        testing::LobbyStep::WaitLobbyRoomGone(name) => {
            done_when(view.status.is_lobby_fetched && !view.status.lobby.contains_key(name))
        }
        testing::LobbyStep::WaitJoinSucceeded(name) => view.join_error.map_or_else(
            || done_when(is_ready_in(name)),
            |reason| testing::E2EStepPoll {
                outcome: testing::StepOutcome::Failed(format!("join failed: {reason}")),
                effect: None,
            },
        ),
        testing::LobbyStep::WaitJoinFailed(name) => {
            done_when(testing::LobbyStep::join_failure_reason(view, name).is_some())
        }
        testing::LobbyStep::WaitLeft(name) => done_when(!is_in(name)),
        testing::LobbyStep::WaitRingJoined => done_when(false),
        testing::LobbyStep::WaitLobbyReady => done_when(view.status.is_lobby_fetched),
        testing::LobbyStep::ExitApp => testing::E2EStepPoll {
            outcome: if clock.elapsed >= testing::QUIT_PUBLISH_GRACE {
                testing::StepOutcome::Done
            } else {
                testing::StepOutcome::Pending
            },
            effect: clock
                .is_first_poll
                .then_some(testing::E2EStepEffect::Extension(
                    discovery::Command::Quit.to_extension_bytes(),
                )),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::poll_lobby_step;
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    const FIRST: testing::StepClock = testing::StepClock {
        elapsed: Duration::ZERO,
        is_first_poll: true,
    };

    fn view(status: &discovery::state_machine::DiscoveryState) -> testing::LobbyStepView<'_> {
        testing::LobbyStepView {
            status,
            join_error: None,
        }
    }

    #[test]
    fn test_usage() {
        let status = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        let room = identity::RoomName::from("room-1");
        let poll = poll_lobby_step(
            &testing::LobbyStep::JoinRoom(room.clone()),
            FIRST,
            &view(&status),
        );
        assert_eq!(poll.outcome, testing::StepOutcome::Done);
        assert!(matches!(
            poll.effect,
            Some(testing::E2EStepEffect::ArmRoomAction(_))
        ));
        let poll = poll_lobby_step(
            &testing::LobbyStep::WaitJoinSucceeded(room),
            FIRST,
            &view(&status),
        );
        assert_eq!(poll.outcome, testing::StepOutcome::Pending);
    }

    #[test]
    fn a_join_error_fails_the_join_wait_at_once() {
        let status = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        let seen = testing::LobbyStepView {
            join_error: Some("room gone"),
            ..view(&status)
        };
        let poll = poll_lobby_step(
            &testing::LobbyStep::WaitJoinSucceeded(identity::RoomName::from("room-1")),
            FIRST,
            &seen,
        );
        assert!(matches!(poll.outcome, testing::StepOutcome::Failed(_)));
    }

    #[test]
    fn a_room_missing_from_the_fetched_lobby_is_a_failed_join() {
        let status = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .lobby_fetched()
            .build();
        let poll = poll_lobby_step(
            &testing::LobbyStep::WaitJoinFailed(identity::RoomName::from("room-1")),
            FIRST,
            &view(&status),
        );
        assert_eq!(poll.outcome, testing::StepOutcome::Done);
    }

    #[test]
    fn exit_app_quits_once_then_waits_the_grace() {
        let status = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        let first = poll_lobby_step(&testing::LobbyStep::ExitApp, FIRST, &view(&status));
        assert_eq!(
            first.effect,
            Some(testing::E2EStepEffect::Extension(
                discovery::Command::Quit.to_extension_bytes()
            ))
        );
        assert_eq!(first.outcome, testing::StepOutcome::Pending);
        let later = testing::StepClock {
            elapsed: Duration::from_secs(3),
            is_first_poll: false,
        };
        let last = poll_lobby_step(&testing::LobbyStep::ExitApp, later, &view(&status));
        assert_eq!(last.effect, None);
        assert_eq!(last.outcome, testing::StepOutcome::Done);
    }

    #[test]
    fn lobby_ready_completes_once_the_lobby_is_fetched() {
        let unfetched = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        let pending = poll_lobby_step(
            &testing::LobbyStep::WaitLobbyReady,
            FIRST,
            &view(&unfetched),
        );
        assert_eq!(pending.outcome, testing::StepOutcome::Pending);
        let fetched = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .lobby_fetched()
            .build();
        let done = poll_lobby_step(&testing::LobbyStep::WaitLobbyReady, FIRST, &view(&fetched));
        assert_eq!(done.outcome, testing::StepOutcome::Done);
    }

    #[test]
    fn ring_joined_stays_pending_without_live_ring_count() {
        let status = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        let poll = poll_lobby_step(&testing::LobbyStep::WaitRingJoined, FIRST, &view(&status));
        assert_eq!(poll.outcome, testing::StepOutcome::Pending);
    }
}
