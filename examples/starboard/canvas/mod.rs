#[path = "__basic__/mod.rs"]
pub mod basic;
mod brush_radius;
mod canvas;
mod decode_snapshot;
mod encode_snapshot;
mod hash_canvas;
mod hash_tile;
mod is_inside_shape;
mod merge_tile;
mod rasterize;
mod read_cell;
mod tile;
mod write_cell;

pub use basic::constants::{
    BASE_BRUSH_RADIUS, BRUSH_RADII, CANVAS_SIDE, CELLS_PER_TILE, DEFAULT_SIZE_INDEX, SIZE_PERCENTS,
    TILE_COUNT, TILE_SIDE, TILES_PER_SIDE,
};
pub use basic::enums::DrawAction;
pub use basic::newtypes::{CanvasSnapshot, TileId};
pub use basic::structs::{Cell, CellPos, DrawOp, TouchedTile};
pub use brush_radius::brush_radius;
pub use canvas::Canvas;
pub use decode_snapshot::decode_snapshot;
pub use encode_snapshot::encode_snapshot;
pub use hash_canvas::hash_canvas;
pub use hash_tile::hash_tile;
pub use is_inside_shape::is_inside_shape;
pub use merge_tile::merge_tile;
pub use rasterize::rasterize;
pub use read_cell::read_cell;
pub use tile::Tile;
pub use write_cell::write_cell;
