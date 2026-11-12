pub use e2e_steps::starboard_steps::{Cell, Stroke};

use crate::starboard;

/// Converts a shared step stroke to the two-point polyline the game draws for it.
#[must_use]
pub fn to_canvas_points(stroke: Stroke) -> Vec<starboard::canvas::SubCellPos> {
    vec![
        starboard::canvas::cell_centre(to_canvas_cell(stroke.from)),
        starboard::canvas::cell_centre(to_canvas_cell(stroke.to)),
    ]
}

/// Converts a shared step cell to the game's canvas cell.
#[must_use]
pub const fn to_canvas_cell(cell: Cell) -> starboard::canvas::CellPos {
    starboard::canvas::CellPos {
        x: cell.x,
        y: cell.y,
    }
}
