use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use crate::testing;

pub fn e2e_follow(
    In(params): In<Option<Value>>,
    mut queue: ResMut<testing::SyntheticInputQueue>,
) -> BrpResult {
    let path: testing::PointerPath =
        serde_json::from_value(params.unwrap_or(Value::Null)).map_err(BrpError::internal)?;
    queue.follow(&path);
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
        let system = app.world_mut().register_system(super::e2e_follow);
        let result = app.world_mut().run_system_with(
            system,
            Some(json!({
                "button": "Left",
                "points": [[0.0, 0.0], [10.0, 0.0]],
                "speed": 10.0
            })),
        );
        assert!(matches!(result, Ok(Ok(Value::Null))));
        let queue = app.world().resource::<testing::SyntheticInputQueue>();
        assert_eq!(queue.pending.len(), 4);
        assert!(matches!(
            queue.pending.front(),
            Some(testing::SyntheticInput::Glide { .. })
        ));
    }
}
