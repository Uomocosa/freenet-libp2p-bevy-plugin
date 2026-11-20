use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Deref, Serialize, Deserialize)]
pub struct DummyMessage(pub u32);
