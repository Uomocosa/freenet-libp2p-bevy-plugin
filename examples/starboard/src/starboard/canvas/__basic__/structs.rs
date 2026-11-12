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

/// A cursor position in 1/`SUBCELL_STEPS` of a cell; a cell's centre is at `cell * SUBCELL_STEPS + SUBCELL_STEPS / 2`.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct SubCellPos {
    pub x: u32,
    pub y: u32,
}

/// The line `a * x + b * y + k >= 0`, used to describe a brush shape as convex pieces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HalfPlane {
    pub a: i64,
    pub b: i64,
    pub k: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TouchedTile {
    pub id: starboard::canvas::TileId,
    pub cells: Vec<Cell>,
}

/// One paint or erase op. `points` is a sub-cell polyline of at least one point: one point is a
/// dab, more points are sampled along a stroke and rasterized pair by pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawOp {
    pub action: starboard::canvas::DrawAction,
    pub shape: starboard::identity::Shape,
    pub size_index: u8,
    pub points: Vec<SubCellPos>,
    pub stroke_rank: u32,
}
