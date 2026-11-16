use freenet_libp2p_bevy_core::testing::KeyStep;

#[derive(Debug, Clone, PartialEq)]
pub enum InputStep {
    ClickAt { x: f32, y: f32 },
    MoveTo { x: f32, y: f32 },
    Keys(KeyStep),
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_core::testing::{Key, KeyStep};

    use super::InputStep;

    #[test]
    fn test_usage() {
        let step = InputStep::ClickAt { x: 120.0, y: 80.0 };
        assert_eq!(step, InputStep::ClickAt { x: 120.0, y: 80.0 });
        assert_ne!(step, InputStep::MoveTo { x: 120.0, y: 80.0 });
        assert_eq!(
            InputStep::Keys(KeyStep::Tap(Key::Tab)),
            InputStep::Keys(KeyStep::Tap(Key::Tab))
        );
        assert_ne!(
            InputStep::Keys(KeyStep::Tap(Key::Tab)),
            InputStep::Keys(KeyStep::Tap(Key::Enter))
        );
    }
}
