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
