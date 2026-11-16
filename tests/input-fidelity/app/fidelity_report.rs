use std::fmt::{Display, Formatter};

use crate::canonical_input::CanonicalInput;

pub struct FidelityReport {
    real: Vec<CanonicalInput>,
    synthetic: Vec<CanonicalInput>,
}

impl FidelityReport {
    #[must_use]
    pub const fn between(real: Vec<CanonicalInput>, synthetic: Vec<CanonicalInput>) -> Self {
        Self { real, synthetic }
    }

    #[must_use]
    pub fn is_match(&self) -> bool {
        self.real == self.synthetic
    }
}

impl Display for FidelityReport {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            formatter,
            "input fidelity: {} records from real XTest, {} from the synthetic driver",
            self.real.len(),
            self.synthetic.len()
        )?;
        for (index, pair) in self.real.iter().zip(self.synthetic.iter()).enumerate() {
            let (real, synthetic) = pair;
            if real != synthetic {
                writeln!(formatter, "index {index}")?;
                writeln!(formatter, "  real:      {real:?}")?;
                writeln!(formatter, "  synthetic: {synthetic:?}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::FidelityReport;
    use crate::canonical_input::CanonicalInput;
    use crate::input_record::{KeyLabel, PressState};
    use bevy::input::keyboard::{Key, KeyCode};

    fn tab_pressed(text: Option<&str>) -> CanonicalInput {
        CanonicalInput::KeyStateChanged {
            physical: KeyLabel::from_key_code(KeyCode::Tab),
            logical: KeyLabel::from_logical_key(&Key::Tab),
            state: PressState::Pressed,
            text: text.map(str::to_string),
        }
    }

    #[test]
    fn test_usage() {
        let matching = FidelityReport::between(
            vec![CanonicalInput::PointerMovedTo { x: 120.0, y: 80.0 }],
            vec![CanonicalInput::PointerMovedTo { x: 120.0, y: 80.0 }],
        );
        assert!(matching.is_match());
        let report =
            FidelityReport::between(vec![tab_pressed(Some("\t"))], vec![tab_pressed(None)]);
        assert!(!report.is_match());
        let rendered = report.to_string();
        assert!(rendered.contains("index 0"));
        assert!(rendered.contains("real:"));
        assert!(rendered.contains("synthetic:"));
    }
}
