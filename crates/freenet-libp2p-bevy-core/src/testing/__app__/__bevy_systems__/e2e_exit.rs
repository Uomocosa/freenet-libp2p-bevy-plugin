use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde_json::Value;

pub fn e2e_exit(In(_params): In<Option<Value>>, mut exits: MessageWriter<AppExit>) -> BrpResult {
    exits.write(AppExit::Success);
    Ok(Value::Null)
}
