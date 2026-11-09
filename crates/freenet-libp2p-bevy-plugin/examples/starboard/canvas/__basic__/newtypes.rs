use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Deref,
)]
pub struct TileId(pub u16);

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Deref)]
pub struct CanvasSnapshot(pub Vec<u8>);
