use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use crate::testing;

pub fn e2e_type_text(
    In(params): In<Option<Value>>,
    mut queue: ResMut<testing::SyntheticInputQueue>,
) -> BrpResult {
    let text: testing::TypedText =
        serde_json::from_value(params.unwrap_or(Value::Null)).map_err(BrpError::internal)?;
    queue.push_text(&text);
    Ok(Value::Null)
}
