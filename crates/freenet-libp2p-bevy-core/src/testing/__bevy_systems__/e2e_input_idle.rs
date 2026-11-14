use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde_json::Value;

use crate::testing;

pub fn e2e_input_idle(
    In(_params): In<Option<Value>>,
    queue: Res<testing::SyntheticInputQueue>,
) -> BrpResult {
    Ok(Value::Bool(queue.into_inner().is_idle()))
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde_json::Value;

    use crate::testing;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.init_resource::<testing::SyntheticInputQueue>();
        let system = app.world_mut().register_system(super::e2e_input_idle);
        let result = app.world_mut().run_system_with(system, None);
        assert!(matches!(result, Ok(Ok(Value::Bool(true)))));
    }
}
