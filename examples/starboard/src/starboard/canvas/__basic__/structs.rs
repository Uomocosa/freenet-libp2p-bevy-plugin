use serde::{Deserialize, Serialize};

use crate::starboard;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct CellPos {
    pub x: u16,
    pub y: u16,
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct SubCellPos {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TouchedTile {
    pub id: starboard::canvas::TileId,
    pub cells: Vec<starboard::canvas::Cell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerKey {
    pub rank: u32,
    pub author: u16,
    pub color: Option<starboard::identity::PaletteColor>,
}
