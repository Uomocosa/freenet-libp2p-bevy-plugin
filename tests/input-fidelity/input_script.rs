use std::sync::LazyLock;

use freenet_libp2p_bevy_core::testing::{Key, KeyStep, Millis, RepeatCount};

use crate::input_step::InputStep;

pub const CLICK_SPEED: f32 = 1200.0;

pub static INPUT_SCRIPT: LazyLock<Vec<InputStep>> = LazyLock::new(|| {
    vec![
        InputStep::ClickAt { x: 120.0, y: 80.0 },
        InputStep::MoveTo { x: 400.0, y: 300.0 },
        InputStep::Keys(KeyStep::Tap(Key::Tab)),
        InputStep::Keys(KeyStep::Text("aB1 ".to_owned())),
        InputStep::Keys(KeyStep::Chord(vec![Key::Ctrl, Key::Char('z')])),
        InputStep::Keys(KeyStep::Tap(Key::Backspace)),
        InputStep::Keys(KeyStep::Down(Key::Char('w'))),
        InputStep::Keys(KeyStep::Wait(Millis(250))),
        InputStep::Keys(KeyStep::Up(Key::Char('w'))),
        InputStep::Keys(KeyStep::Tap(Key::Enter)),
        InputStep::Keys(KeyStep::Tap(Key::Escape)),
        InputStep::ClickAt { x: 300.0, y: 220.0 },
    ]
});

pub static SYNTHETIC_ONLY_SCRIPT: LazyLock<Vec<InputStep>> = LazyLock::new(|| {
    vec![
        InputStep::Keys(KeyStep::Repeat {
            key: Key::ArrowRight,
            count: RepeatCount(3),
        }),
        InputStep::Keys(KeyStep::SameFrame(vec![
            KeyStep::Tap(Key::Char(' ')),
            KeyStep::Tap(Key::Char(' ')),
        ])),
    ]
});

#[cfg(test)]
mod tests {
    use super::{INPUT_SCRIPT, InputStep, SYNTHETIC_ONLY_SCRIPT};

    #[test]
    fn test_usage() {
        assert_eq!(INPUT_SCRIPT.len(), 12);
        assert!(INPUT_SCRIPT.contains(&InputStep::Keys(
            freenet_libp2p_bevy_core::testing::KeyStep::Text("aB1 ".to_owned())
        )));
        assert_eq!(SYNTHETIC_ONLY_SCRIPT.len(), 2);
        let mut points: Vec<(f32, f32)> = Vec::new();
        for step in INPUT_SCRIPT.iter().chain(SYNTHETIC_ONLY_SCRIPT.iter()) {
            match step {
                InputStep::ClickAt { x, y } | InputStep::MoveTo { x, y } => {
                    assert!(!points.contains(&(*x, *y)), "every pointer step sits alone");
                    points.push((*x, *y));
                }
                InputStep::Keys(_) => {}
            }
        }
    }
}
