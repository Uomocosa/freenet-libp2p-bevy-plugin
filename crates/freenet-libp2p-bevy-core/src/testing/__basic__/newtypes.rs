use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deref, Serialize, Deserialize)]
pub struct PointerSpeed(pub f32);

#[derive(Debug, Clone, PartialEq, Eq, Deref, Deserialize)]
#[serde(transparent)]
pub struct TypedText(pub String);
