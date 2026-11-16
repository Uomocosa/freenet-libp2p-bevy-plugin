use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde_json::Value;

use crate::input_recorder::InputRecorder;

pub fn probe_clear(
    In(_params): In<Option<Value>>,
    mut recorder: ResMut<InputRecorder>,
) -> BrpResult {
    recorder.clear();
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde_json::Value;

    use super::probe_clear;
    use crate::input_record::{InputRecord, MouseButtonKind, PressState};
    use crate::input_recorder::InputRecorder;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let mut recorder = InputRecorder::default();
        recorder.records.push(InputRecord::MouseButton {
            button: MouseButtonKind::Left,
            state: PressState::Released,
        });
        app.insert_resource(recorder);
        let system = app.world_mut().register_system(probe_clear);
        let result = app.world_mut().run_system_with(system, None::<Value>);
        let Ok(Ok(value)) = result else {
            panic!("probe_clear did not run: {result:?}");
        };
        assert_eq!(value, Value::Null);
        assert_eq!(app.world().resource::<InputRecorder>().records.len(), 0);
    }
}
