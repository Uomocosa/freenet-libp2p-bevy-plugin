use bevy::prelude::Resource;
use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Deref, Resource,
)]
pub struct NetworkId(pub u64);
