#[path = "../../../../tests/e2e/tests/starboard_steps/mod.rs"]
pub mod starboard_steps;

pub use starboard_steps::{Cell, StarboardStep, Stroke};

use crate::starboard;

/// Converts a shared step stroke to the game's canvas stroke.
#[must_use]
pub const fn to_canvas_stroke(stroke: Stroke) -> starboard::canvas::Stroke {
    starboard::canvas::Stroke {
        from: to_canvas_cell(stroke.from),
        to: to_canvas_cell(stroke.to),
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
