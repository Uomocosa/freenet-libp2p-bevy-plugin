use freenet_libp2p_bevy_core::testing::NamedKey;

use crate::input_step::InputStep;

pub const CLICK_SPEED: f32 = 1200.0;

pub const INPUT_SCRIPT: &[InputStep] = &[
    InputStep::ClickAt { x: 120.0, y: 80.0 },
    InputStep::MoveTo { x: 400.0, y: 300.0 },
    InputStep::PressKey(NamedKey::Tab),
    InputStep::TypeText("aB1 "),
    InputStep::PressKey(NamedKey::Backspace),
    InputStep::PressKey(NamedKey::Enter),
    InputStep::PressKey(NamedKey::Escape),
    InputStep::ClickAt { x: 300.0, y: 220.0 },
];

#[cfg(test)]
mod tests {
    use super::{INPUT_SCRIPT, InputStep};

    #[test]
    fn test_usage() {
        assert_eq!(INPUT_SCRIPT.len(), 8);
        assert!(INPUT_SCRIPT.contains(&InputStep::TypeText("aB1 ")));
        let mut points: Vec<(f32, f32)> = Vec::new();
        for step in INPUT_SCRIPT {
            match step {
                InputStep::ClickAt { x, y } | InputStep::MoveTo { x, y } => {
                    assert!(!points.contains(&(*x, *y)), "every pointer step sits alone");
                    points.push((*x, *y));
                }
                InputStep::PressKey(_) | InputStep::TypeText(_) => {}
            }
        }
    }
}
