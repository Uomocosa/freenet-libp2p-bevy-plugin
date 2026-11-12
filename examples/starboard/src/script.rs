pub use e2e_steps::starboard_steps::{Cell, Stroke};

use crate::starboard;

/// Converts a shared step stroke to the game's canvas stroke.
#[must_use]
pub fn to_canvas_stroke(stroke: Stroke) -> starboard::canvas::Stroke {
    starboard::canvas::Stroke {
        from: starboard::canvas::cell_centre(to_canvas_cell(stroke.from)),
        to: starboard::canvas::cell_centre(to_canvas_cell(stroke.to)),
    }
}

/// Converts a shared step cell to the game's canvas cell.
#[must_use]
pub const fn to_canvas_cell(cell: Cell) -> starboard::canvas::CellPos {
    starboard::canvas::CellPos {
        x: cell.x,
        y: cell.y,
    }
}
