#[path = "__basic__/mod.rs"]
mod basic;
mod brush_radius;
mod canvas;
mod cell;
mod cell_box;
mod cell_centre;
mod cell_coverage;
mod decode_snapshot;
mod draw_op;
mod encode_snapshot;
mod half_plane;
mod hash_canvas;
mod hash_tile;
mod is_inside_shape;
mod layer;
mod merge_tile;
mod pair_bins;
mod rasterize;
mod read_cell;
mod segment_meets_shape;
mod segment_sweep;
mod shape_pieces;
mod stroke_raster;
mod tile;
mod under_layer;
mod write_cell;

pub use basic::constants::{
    BASE_BRUSH_RADIUS, BRUSH_RADII, CANVAS_SIDE, CELLS_PER_TILE, DEFAULT_SIZE_INDEX, FULL_COVERAGE,
    MAX_POINTS_PER_OP, SAMPLE_OFFSETS, SAMPLES_PER_CELL, SIZE_PERCENTS, SUBCELL_STEPS, TILE_COUNT,
    TILE_SIDE, TILES_PER_SIDE, VERTEX_SCALE,
};
pub use basic::enums::DrawAction;
pub use basic::newtypes::{CanvasSnapshot, TileId};
pub use basic::structs::{CellPos, LayerKey, SubCellPos, TouchedTile};
pub use brush_radius::brush_radius;
pub use canvas::Canvas;
pub use cell::Cell;
pub use cell_box::CellBox;
pub use cell_centre::cell_centre;
pub use cell_coverage::CellCoverage;
pub use decode_snapshot::decode_snapshot;
pub use draw_op::DrawOp;
pub use encode_snapshot::encode_snapshot;
pub use half_plane::HalfPlane;
pub use hash_canvas::hash_canvas;
pub use hash_tile::hash_tile;
pub use is_inside_shape::is_inside_shape;
pub use layer::Layer;
pub use merge_tile::merge_tile;
pub use pair_bins::PairBins;
pub use rasterize::rasterize;
pub use read_cell::read_cell;
pub use segment_meets_shape::segment_meets_shape;
pub use segment_sweep::SegmentSweep;
pub use shape_pieces::shape_pieces;
pub use stroke_raster::StrokeRaster;
pub use tile::Tile;
pub use under_layer::UnderLayer;
pub use write_cell::write_cell;
