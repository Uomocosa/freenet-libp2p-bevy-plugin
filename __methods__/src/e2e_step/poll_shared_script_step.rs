use crate::discovery;
use crate::identity;
use crate::testing;

#[must_use]
pub fn poll_shared_script_step(
    step: &testing::E2EStep,
    clock: testing::StepClock,
    view: &testing::E2EStepView<'_>,
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
        testing::E2EStep::CreateRoom(name) => testing::E2EStepPoll {
            outcome: testing::StepOutcome::Done,
            effect: Some(testing::E2EStepEffect::ArmRoomAction(
                testing::Action::Create(name.clone()),
            )),
        },
        testing::E2EStep::JoinRoom(name) => testing::E2EStepPoll {
            outcome: testing::StepOutcome::Done,
            effect: Some(testing::E2EStepEffect::ArmRoomAction(
                testing::Action::Join(name.clone()),
            )),
        },
        testing::E2EStep::LeaveRoom(_) => testing::E2EStepPoll {
            outcome: testing::StepOutcome::Done,
            effect: Some(testing::E2EStepEffect::SendDiscoveryCommand(
                discovery::Command::Leave,
            )),
        },
        testing::E2EStep::WaitLobbyRoom(name) => done_when(view.status.lobby.contains_key(name)),
        testing::E2EStep::WaitLobbyRoomGone(name) => {
            done_when(view.status.is_lobby_fetched && !view.status.lobby.contains_key(name))
        }
        testing::E2EStep::WaitJoinSucceeded(name) => view.join_error.map_or_else(
            || done_when(is_ready_in(name)),
            |reason| testing::E2EStepPoll {
                outcome: testing::StepOutcome::Failed(format!("join failed: {reason}")),
                effect: None,
            },
        ),
        testing::E2EStep::WaitJoinFailed(name) => {
            done_when(testing::E2EStep::join_failure_reason(view, name).is_some())
        }
        testing::E2EStep::WaitLeft(name) => done_when(!is_in(name)),
        testing::E2EStep::WaitMembers(count) => {
            let members = view.status.connected_members().count().saturating_add(1);
            done_when(u32::try_from(members).unwrap_or(u32::MAX) >= **count)
        }
        testing::E2EStep::WaitMembersAreStrictlyLessThan(count) => {
            let members = view.status.connected_members().count().saturating_add(1);
            done_when(u32::try_from(members).unwrap_or(u32::MAX) < **count)
        }
        testing::E2EStep::WaitMembersAreLessOrEqualTo(count) => {
            let members = view.status.connected_members().count().saturating_add(1);
            done_when(u32::try_from(members).unwrap_or(u32::MAX) <= **count)
        }
        testing::E2EStep::WaitRoomMessages(count) => {
            done_when(u32::try_from(view.room_message_senders).unwrap_or(u32::MAX) >= **count)
        }
        testing::E2EStep::WaitRingJoined | testing::E2EStep::Stay => done_when(false),
        testing::E2EStep::WaitLobbyReady => done_when(view.status.is_lobby_fetched),
        testing::E2EStep::Sleep(duration) => done_when(clock.elapsed >= *duration),
        testing::E2EStep::MarkCompleted(checkpoint) => {
            tracing::info!(target: "script", "script checkpoint completed name={checkpoint}");
            testing::E2EStepPoll {
                outcome: testing::StepOutcome::Done,
                effect: None,
            }
        }
        testing::E2EStep::ExitApp => {
            testing::E2EStepPoll {
                outcome: if clock.elapsed >= testing::QUIT_PUBLISH_GRACE {
                    testing::StepOutcome::Done
                } else {
                    testing::StepOutcome::Pending
                },
                effect: clock.is_first_poll.then_some(
                    testing::E2EStepEffect::SendDiscoveryCommand(discovery::Command::Quit),
                ),
            }
        }
        testing::E2EStep::CrashApp => testing::E2EStepPoll {
            outcome: testing::StepOutcome::Pending,
            effect: Some(testing::E2EStepEffect::AbortProcess),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::poll_shared_script_step;
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    const FIRST: testing::StepClock = testing::StepClock {
        elapsed: Duration::ZERO,
        is_first_poll: true,
    };

    fn view(status: &discovery::Status) -> testing::E2EStepView<'_> {
        testing::E2EStepView {
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
            &testing::E2EStep::JoinRoom(room.clone()),
            FIRST,
            &view(&status),
        );
        assert_eq!(poll.outcome, testing::StepOutcome::Done);
        assert!(matches!(
            poll.effect,
            Some(testing::E2EStepEffect::ArmRoomAction(_))
        ));
        let poll = poll_shared_script_step(
            &testing::E2EStep::WaitJoinSucceeded(room),
            FIRST,
            &view(&status),
        );
        assert_eq!(poll.outcome, testing::StepOutcome::Pending);
    }

    #[test]
    fn a_join_error_fails_the_join_wait_at_once() {
        let status = discovery::Status::default();
        let seen = testing::E2EStepView {
            join_error: Some("room gone"),
            ..view(&status)
        };
        let poll = poll_shared_script_step(
            &testing::E2EStep::WaitJoinSucceeded(identity::RoomName::from("room-1")),
            FIRST,
            &seen,
        );
        assert!(matches!(poll.outcome, testing::StepOutcome::Failed(_)));
    }

    #[test]
    fn a_room_missing_from_the_fetched_lobby_is_a_failed_join() {
        let status = discovery::Status {
            is_lobby_fetched: true,
            ..discovery::Status::default()
        };
        let poll = poll_shared_script_step(
            &testing::E2EStep::WaitJoinFailed(identity::RoomName::from("room-1")),
            FIRST,
            &view(&status),
        );
        assert_eq!(poll.outcome, testing::StepOutcome::Done);
    }

    #[test]
    fn exit_app_quits_once_then_waits_the_grace() {
        let status = discovery::Status::default();
        let first = poll_shared_script_step(&testing::E2EStep::ExitApp, FIRST, &view(&status));
        assert_eq!(
            first.effect,
            Some(testing::E2EStepEffect::SendDiscoveryCommand(
                discovery::Command::Quit
            ))
        );
        assert_eq!(first.outcome, testing::StepOutcome::Pending);
        let later = testing::StepClock {
            elapsed: Duration::from_secs(3),
            is_first_poll: false,
        };
        let last = poll_shared_script_step(&testing::E2EStep::ExitApp, later, &view(&status));
        assert_eq!(last.effect, None);
        assert_eq!(last.outcome, testing::StepOutcome::Done);
    }

    #[test]
    fn strictly_less_member_wait_completes_below_the_bound() {
        let status = discovery::Status::default();
        let alone = view(&status);
        let done = poll_shared_script_step(
            &testing::E2EStep::WaitMembersAreStrictlyLessThan(testing::MemberCount(2)),
            FIRST,
            &alone,
        );
        assert_eq!(done.outcome, testing::StepOutcome::Done);
        let pending = poll_shared_script_step(
            &testing::E2EStep::WaitMembersAreStrictlyLessThan(testing::MemberCount(1)),
            FIRST,
            &alone,
        );
        assert_eq!(pending.outcome, testing::StepOutcome::Pending);
    }

    #[test]
    fn less_or_equal_member_wait_completes_at_the_bound() {
        let status = discovery::Status::default();
        let alone = view(&status);
        let done = poll_shared_script_step(
            &testing::E2EStep::WaitMembersAreLessOrEqualTo(testing::MemberCount(1)),
            FIRST,
            &alone,
        );
        assert_eq!(done.outcome, testing::StepOutcome::Done);
        let pending = poll_shared_script_step(
            &testing::E2EStep::WaitMembersAreLessOrEqualTo(testing::MemberCount(0)),
            FIRST,
            &alone,
        );
        assert_eq!(pending.outcome, testing::StepOutcome::Pending);
    }

    #[test]
    fn lobby_ready_completes_once_the_lobby_is_fetched() {
        let unfetched = discovery::Status::default();
        let pending =
            poll_shared_script_step(&testing::E2EStep::WaitLobbyReady, FIRST, &view(&unfetched));
        assert_eq!(pending.outcome, testing::StepOutcome::Pending);
        let fetched = discovery::Status {
            is_lobby_fetched: true,
            ..discovery::Status::default()
        };
        let done =
            poll_shared_script_step(&testing::E2EStep::WaitLobbyReady, FIRST, &view(&fetched));
        assert_eq!(done.outcome, testing::StepOutcome::Done);
    }

    #[test]
    fn ring_joined_stays_pending_without_live_ring_count() {
        let status = discovery::Status::default();
        let poll =
            poll_shared_script_step(&testing::E2EStep::WaitRingJoined, FIRST, &view(&status));
        assert_eq!(poll.outcome, testing::StepOutcome::Pending);
    }

    #[test]
    fn checkpoint_marker_survives_a_script_info_only_subscriber() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone, Debug)]
        struct SharedBuffer {
            bytes: Arc<Mutex<Vec<u8>>>,
        }

        impl std::io::Write for SharedBuffer {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.bytes.lock().expect("buffer").write(buf)
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for SharedBuffer {
            type Writer = Self;

            fn make_writer(&'a self) -> Self::Writer {
                self.clone()
            }
        }

        let buffer = SharedBuffer {
            bytes: Arc::new(Mutex::new(Vec::new())),
        };
        let filter = tracing_subscriber::EnvFilter::try_new("script=info").expect("filter");
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(buffer.clone())
            .with_ansi(false)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            let status = discovery::Status::default();
            let poll = poll_shared_script_step(
                &testing::E2EStep::MarkCompleted(testing::ScriptCheckpoint::new("probe")),
                FIRST,
                &view(&status),
            );
            assert_eq!(poll.outcome, testing::StepOutcome::Done);
        });
        let text = String::from_utf8(buffer.bytes.lock().expect("buffer").clone()).expect("utf8");
        assert!(
            text.contains("script checkpoint completed name=probe"),
            "checkpoint marker missing under script=info: {text}"
        );
    }
}
