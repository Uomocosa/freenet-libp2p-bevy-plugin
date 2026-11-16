use crate::testing;

pub fn push_same_frame(
    queue: &mut testing::SyntheticInputQueue,
    steps: &[testing::KeyStep],
) -> Result<(), testing::KeyStepError> {
    let mut scratch = testing::SyntheticInputQueue::default();
    scratch.push_steps(steps)?;
    let mut events = Vec::new();
    for item in scratch.pending {
        match item {
            testing::SyntheticInput::Keys(nested) => events.extend(nested),
            _ => return Err(testing::KeyStepError::WaitInSameFrame),
        }
    }
    queue
        .pending
        .push_back(testing::SyntheticInput::Keys(events));
    Ok(())
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;

    use crate::testing;

    #[test]
    fn test_usage() {
        let mut queue = testing::SyntheticInputQueue::default();
        super::push_same_frame(
            &mut queue,
            &[
                testing::KeyStep::Tap(testing::Key::Char(' ')),
                testing::KeyStep::Tap(testing::Key::Char(' ')),
            ],
        )
        .expect("two taps share one frame");
        assert_eq!(queue.pending.len(), 1);
        assert!(matches!(
            &queue.pending[0],
            testing::SyntheticInput::Keys(events)
                if events.len() == 4
                    && events[0].key == testing::Key::Char(' ')
                    && events[0].state == ButtonState::Pressed
                    && events[1].state == ButtonState::Released
                    && events[2].state == ButtonState::Pressed
                    && events[3].state == ButtonState::Released
        ));
    }

    #[test]
    fn test_wait_inside_same_frame_is_rejected() {
        let mut queue = testing::SyntheticInputQueue::default();
        let outcome =
            super::push_same_frame(&mut queue, &[testing::KeyStep::Wait(testing::Millis(60))]);
        assert!(matches!(
            outcome,
            Err(testing::KeyStepError::WaitInSameFrame)
        ));
        assert!(queue.pending.is_empty());
    }

    #[test]
    fn test_nested_same_frame_flattens_into_one_frame() {
        let mut queue = testing::SyntheticInputQueue::default();
        super::push_same_frame(
            &mut queue,
            &[
                testing::KeyStep::Down(testing::Key::Char('a')),
                testing::KeyStep::SameFrame(vec![testing::KeyStep::Up(testing::Key::Char('a'))]),
            ],
        )
        .expect("nested frames flatten");
        assert_eq!(queue.pending.len(), 1);
        assert!(matches!(
            &queue.pending[0],
            testing::SyntheticInput::Keys(events)
                if events.len() == 2
                    && events[0].state == ButtonState::Pressed
                    && events[1].state == ButtonState::Released
        ));
    }
}
