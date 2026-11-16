use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use crate::input_recorder::InputRecorder;

pub fn probe_records(In(_params): In<Option<Value>>, recorder: Res<InputRecorder>) -> BrpResult {
    serde_json::to_value(&recorder.into_inner().records).map_err(BrpError::internal)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde_json::Value;

    use super::probe_records;
    use crate::input_record::{InputRecord, MouseButtonKind, PressState};
    use crate::input_recorder::InputRecorder;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let mut recorder = InputRecorder::default();
        recorder.records.push(InputRecord::MouseButton {
            button: MouseButtonKind::Left,
            state: PressState::Pressed,
        });
        app.insert_resource(recorder);
        let system = app.world_mut().register_system(probe_records);
        let result = app.world_mut().run_system_with(system, None::<Value>);
        let Ok(Ok(value)) = result else {
            panic!("probe_records did not run: {result:?}");
        };
        assert_eq!(value.as_array().map(Vec::len), Some(1));
    }
}
