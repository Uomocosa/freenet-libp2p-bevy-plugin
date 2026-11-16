use freenet_libp2p_bevy_core::testing::Key;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputStep {
    ClickAt { x: f32, y: f32 },
    MoveTo { x: f32, y: f32 },
    PressKey(Key),
    TypeText(&'static str),
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_core::testing::Key;

    use super::InputStep;

    #[test]
    fn test_usage() {
        let step = InputStep::ClickAt { x: 120.0, y: 80.0 };
        assert_eq!(step, InputStep::ClickAt { x: 120.0, y: 80.0 });
        assert_ne!(step, InputStep::MoveTo { x: 120.0, y: 80.0 });
        assert_eq!(InputStep::PressKey(Key::Tab), InputStep::PressKey(Key::Tab));
        assert_eq!(InputStep::TypeText("aB1 "), InputStep::TypeText("aB1 "));
    }
}
