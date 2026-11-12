#[path = "__basic__/mod.rs"]
mod basic;
mod brush_radius;
mod canvas;
mod cell_box;
mod cell_centre;
mod decode_snapshot;
mod drop_collinear_points;
mod encode_snapshot;
mod hash_canvas;
mod hash_tile;
mod is_inside_shape;
mod merge_tile;
mod pair_bins;
mod point_pairs;
mod rasterize;
mod read_cell;
mod segment_meets_shape;
mod shape_pieces;
mod tile;
mod write_cell;

pub use basic::constants::{
    BASE_BRUSH_RADIUS, BRUSH_RADII, CANVAS_SIDE, CELLS_PER_TILE, DEFAULT_SIZE_INDEX, SIZE_PERCENTS,
    SUBCELL_STEPS, TILE_COUNT, TILE_SIDE, TILES_PER_SIDE, VERTEX_SCALE,
};
pub use basic::enums::DrawAction;
pub use basic::newtypes::{CanvasSnapshot, TileId};
pub use basic::structs::{Cell, CellPos, DrawOp, HalfPlane, SubCellPos, TouchedTile};
pub use brush_radius::brush_radius;
pub use canvas::Canvas;
pub use cell_box::CellBox;
pub use cell_centre::cell_centre;
pub use decode_snapshot::decode_snapshot;
pub use drop_collinear_points::drop_collinear_points;
pub use encode_snapshot::encode_snapshot;
pub use hash_canvas::hash_canvas;
pub use hash_tile::hash_tile;
pub use is_inside_shape::is_inside_shape;
pub use merge_tile::merge_tile;
pub use pair_bins::PairBins;
pub use point_pairs::point_pairs;
pub use rasterize::rasterize;
pub use read_cell::read_cell;
pub use segment_meets_shape::segment_meets_shape;
pub use shape_pieces::shape_pieces;
pub use tile::Tile;
pub use write_cell::write_cell;
