use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deref, Serialize, Deserialize)]
pub struct PointerSpeed(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, From, Serialize, Deserialize)]
#[from(forward)]
pub struct Millis(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref, From, Serialize, Deserialize)]
#[from(forward)]
pub struct RepeatCount(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Deref, Deserialize)]
#[serde(transparent)]
pub struct TypedText(pub String);
