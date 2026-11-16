use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde_json::Value;

use crate::input_recorder::InputRecorder;

pub const READY_FRAMES: u32 = 30;

pub fn probe_ready(In(_params): In<Option<Value>>, recorder: Res<InputRecorder>) -> BrpResult {
    Ok(Value::Bool(
        recorder.into_inner().frames_ran >= READY_FRAMES,
    ))
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde_json::Value;

    use super::{READY_FRAMES, probe_ready};
    use crate::input_recorder::InputRecorder;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let recorder = InputRecorder {
            frames_ran: READY_FRAMES,
            ..Default::default()
        };
        app.insert_resource(recorder);
        let system = app.world_mut().register_system(probe_ready);
        let result = app.world_mut().run_system_with(system, None::<Value>);
        let Ok(Ok(value)) = result else {
            panic!("probe_ready did not run: {result:?}");
        };
        assert_eq!(value, Value::Bool(true));
    }
}
