use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use crate::testing;

pub fn e2e_keys(
    In(params): In<Option<Value>>,
    mut queue: ResMut<testing::SyntheticInputQueue>,
) -> BrpResult {
    let steps: Vec<testing::KeyStep> =
        serde_json::from_value(params.unwrap_or(Value::Null)).map_err(BrpError::internal)?;
    queue.push_steps(&steps).map_err(BrpError::internal)?;
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde_json::Value;
    use serde_json::json;

    use crate::testing;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.init_resource::<testing::SyntheticInputQueue>();
        let system = app.world_mut().register_system(super::e2e_keys);
        let result = app
            .world_mut()
            .run_system_with(system, Some(json!([{"Tap": "Enter"}, {"Wait": 250}])));
        assert!(matches!(result, Ok(Ok(Value::Null))));
        let queue = app.world().resource::<testing::SyntheticInputQueue>();
        assert_eq!(queue.pending.len(), 3);
        assert_eq!(
            queue.pending[2],
            testing::SyntheticInput::Wait(testing::Millis(250))
        );
    }

    #[test]
    fn test_wait_inside_same_frame_is_rejected() {
        let mut app = App::new();
        app.init_resource::<testing::SyntheticInputQueue>();
        let system = app.world_mut().register_system(super::e2e_keys);
        let result = app
            .world_mut()
            .run_system_with(system, Some(json!([{"SameFrame": [{"Wait": 60}]}])));
        assert!(matches!(result, Ok(Err(_))));
        assert!(
            app.world()
                .resource::<testing::SyntheticInputQueue>()
                .pending
                .is_empty()
        );
    }
}
