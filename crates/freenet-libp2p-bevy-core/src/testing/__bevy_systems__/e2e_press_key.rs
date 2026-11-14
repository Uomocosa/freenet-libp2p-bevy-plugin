use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use crate::testing;

pub fn e2e_press_key(
    In(params): In<Option<Value>>,
    mut queue: ResMut<testing::SyntheticInputQueue>,
) -> BrpResult {
    let key: testing::NamedKey =
        serde_json::from_value(params.unwrap_or(Value::Null)).map_err(BrpError::internal)?;
    queue.press_named_key(key);
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
        let system = app.world_mut().register_system(super::e2e_press_key);
        let result = app
            .world_mut()
            .run_system_with(system, Some(json!("Escape")));
        assert!(matches!(result, Ok(Ok(Value::Null))));
        assert_eq!(
            app.world()
                .resource::<testing::SyntheticInputQueue>()
                .pending
                .len(),
            2
        );
    }
}
