use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde_json::Value;

pub fn e2e_exit(In(_params): In<Option<Value>>, mut exits: MessageWriter<AppExit>) -> BrpResult {
    exits.write(AppExit::Success);
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde_json::Value;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<AppExit>();
        let system = app.world_mut().register_system(super::e2e_exit);
        let result = app.world_mut().run_system_with(system, None);
        assert!(matches!(result, Ok(Ok(Value::Null))));
        assert_eq!(app.world().resource::<Messages<AppExit>>().len(), 1);
    }
}
