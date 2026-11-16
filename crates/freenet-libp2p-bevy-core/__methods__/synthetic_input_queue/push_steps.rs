use crate::testing;

pub fn push_steps(
    queue: &mut testing::SyntheticInputQueue,
    steps: &[testing::KeyStep],
) -> Result<(), testing::KeyStepError> {
    for step in steps {
        match step {
            testing::KeyStep::Text(text) => queue.push_text(text),
            testing::KeyStep::Tap(key) => queue.push_tap(*key),
            testing::KeyStep::Down(key) => queue.push_shifted_down(*key),
            testing::KeyStep::Up(key) => queue.push_shifted_up(*key),
            testing::KeyStep::Chord(keys) => queue.push_chord(keys),
            testing::KeyStep::Repeat { key, count } => queue.push_repeat(*key, *count),
            testing::KeyStep::Wait(millis) => queue
                .pending
                .push_back(testing::SyntheticInput::Wait(*millis)),
            testing::KeyStep::SameFrame(steps) => queue.push_same_frame(steps)?,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;

    use crate::testing;

    #[test]
    fn test_usage() {
        let mut queue = testing::SyntheticInputQueue::default();
        super::push_steps(
            &mut queue,
            &[
                testing::KeyStep::Tap(testing::Key::Enter),
                testing::KeyStep::Wait(testing::Millis(250)),
            ],
        )
        .expect("tap and wait expand");
        assert_eq!(queue.pending.len(), 3);
        assert!(matches!(queue.pending[0], testing::SyntheticInput::Keys(_)));
        assert_eq!(
            queue.pending[2],
            testing::SyntheticInput::Wait(testing::Millis(250))
        );
    }

    #[test]
    fn test_chord_presses_in_order_and_releases_reversed() {
        let mut queue = testing::SyntheticInputQueue::default();
        super::push_steps(
            &mut queue,
            &[testing::KeyStep::Chord(vec![
                testing::Key::Ctrl,
                testing::Key::Char('z'),
            ])],
        )
        .expect("chord expands");
        assert_eq!(queue.pending.len(), 4);
        assert!(matches!(
            &queue.pending[0],
            testing::SyntheticInput::Keys(events)
                if events.len() == 1
                    && events[0].key == testing::Key::Ctrl
                    && events[0].state == ButtonState::Pressed
        ));
        assert!(matches!(
            &queue.pending[1],
            testing::SyntheticInput::Keys(events)
                if events.len() == 1
                    && events[0].key == testing::Key::Char('z')
                    && events[0].state == ButtonState::Pressed
                    && events[0].does_emit_text
        ));
        assert!(matches!(
            &queue.pending[2],
            testing::SyntheticInput::Keys(events)
                if events.len() == 1
                    && events[0].key == testing::Key::Char('z')
                    && events[0].state == ButtonState::Released
        ));
        assert!(matches!(
            &queue.pending[3],
            testing::SyntheticInput::Keys(events)
                if events.len() == 1
                    && events[0].key == testing::Key::Ctrl
                    && events[0].state == ButtonState::Released
        ));
    }

    #[test]
    fn test_wait_inside_same_frame_is_rejected() {
        let mut queue = testing::SyntheticInputQueue::default();
        let outcome = super::push_steps(
            &mut queue,
            &[testing::KeyStep::SameFrame(vec![testing::KeyStep::Wait(
                testing::Millis(60),
            )])],
        );
        assert!(matches!(
            outcome,
            Err(testing::KeyStepError::WaitInSameFrame)
        ));
    }
}
