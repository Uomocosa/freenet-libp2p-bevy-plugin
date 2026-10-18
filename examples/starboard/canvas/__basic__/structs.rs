use serde::{Deserialize, Serialize};

use crate::starboard;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Cell {
    pub lamport: u32,
    pub author: u16,
    pub color: Option<starboard::identity::PaletteColor>,
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct CellPos {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TouchedTile {
    pub id: starboard::canvas::TileId,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawOp {
    pub action: starboard::canvas::DrawAction,
    pub shape: starboard::identity::Shape,
    pub size_index: u8,
    pub from: CellPos,
    pub to: CellPos,
}
