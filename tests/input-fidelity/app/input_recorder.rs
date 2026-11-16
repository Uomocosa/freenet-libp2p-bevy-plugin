use bevy::prelude::*;

use crate::input_record::InputRecord;

#[derive(Resource, Debug, Default)]
pub struct InputRecorder {
    pub records: Vec<InputRecord>,
    pub frames_ran: u32,
}

impl InputRecorder {
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::InputRecorder;
    use crate::input_record::{MouseButtonKind, PressState};

    #[test]
    fn test_usage() {
        let mut recorder = InputRecorder::default();
        recorder
            .records
            .push(crate::input_record::InputRecord::MouseButton {
                button: MouseButtonKind::Left,
                state: PressState::Pressed,
            });
        assert_eq!(recorder.records.len(), 1);
        recorder.clear();
        assert_eq!(recorder.records.len(), 0);
    }
}
