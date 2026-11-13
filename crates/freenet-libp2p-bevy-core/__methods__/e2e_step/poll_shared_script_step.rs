use crate::testing;

#[must_use]
pub fn poll_shared_script_step(
    step: &testing::E2EStep,
    clock: testing::StepClock,
    view: &testing::E2EStepView,
) -> testing::E2EStepPoll {
    let done_when = |is_done: bool| testing::E2EStepPoll {
        outcome: if is_done {
            testing::StepOutcome::Done
        } else {
            testing::StepOutcome::Pending
        },
        effect: None,
    };
    let members = u32::try_from(view.members).unwrap_or(u32::MAX);
    match step {
        testing::E2EStep::WaitMembers(count) => done_when(members >= **count),
        testing::E2EStep::WaitMembersAreStrictlyLessThan(count) => done_when(members < **count),
        testing::E2EStep::WaitMembersAreLessOrEqualTo(count) => done_when(members <= **count),
        testing::E2EStep::WaitRoomMessages(count) => {
            done_when(u32::try_from(view.room_message_senders).unwrap_or(u32::MAX) >= **count)
        }
        testing::E2EStep::Stay => done_when(false),
        testing::E2EStep::Sleep(duration) => done_when(clock.elapsed >= *duration),
        testing::E2EStep::MarkCompleted(checkpoint) => {
            tracing::info!(target: "script", "script checkpoint completed name={checkpoint}");
            testing::E2EStepPoll {
                outcome: testing::StepOutcome::Done,
                effect: None,
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
    use crate::testing;

    const FIRST: testing::StepClock = testing::StepClock {
        elapsed: Duration::ZERO,
        is_first_poll: true,
    };

    fn view(members: usize) -> testing::E2EStepView {
        testing::E2EStepView {
            room_message_senders: 0,
            members,
        }
    }

    #[test]
    fn test_usage() {
        let pending = poll_shared_script_step(
            &testing::E2EStep::WaitMembers(testing::MemberCount(2)),
            FIRST,
            &view(1),
        );
        assert_eq!(pending.outcome, testing::StepOutcome::Pending);
        let done = poll_shared_script_step(
            &testing::E2EStep::WaitMembers(testing::MemberCount(2)),
            FIRST,
            &view(2),
        );
        assert_eq!(done.outcome, testing::StepOutcome::Done);
    }

    #[test]
    fn strictly_less_member_wait_completes_below_the_bound() {
        let done = poll_shared_script_step(
            &testing::E2EStep::WaitMembersAreStrictlyLessThan(testing::MemberCount(2)),
            FIRST,
            &view(1),
        );
        assert_eq!(done.outcome, testing::StepOutcome::Done);
        let pending = poll_shared_script_step(
            &testing::E2EStep::WaitMembersAreStrictlyLessThan(testing::MemberCount(1)),
            FIRST,
            &view(1),
        );
        assert_eq!(pending.outcome, testing::StepOutcome::Pending);
    }

    #[test]
    fn less_or_equal_member_wait_completes_at_the_bound() {
        let done = poll_shared_script_step(
            &testing::E2EStep::WaitMembersAreLessOrEqualTo(testing::MemberCount(1)),
            FIRST,
            &view(1),
        );
        assert_eq!(done.outcome, testing::StepOutcome::Done);
        let pending = poll_shared_script_step(
            &testing::E2EStep::WaitMembersAreLessOrEqualTo(testing::MemberCount(0)),
            FIRST,
            &view(1),
        );
        assert_eq!(pending.outcome, testing::StepOutcome::Pending);
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
            let poll = poll_shared_script_step(
                &testing::E2EStep::MarkCompleted(testing::ScriptCheckpoint::new("probe")),
                FIRST,
                &view(1),
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
