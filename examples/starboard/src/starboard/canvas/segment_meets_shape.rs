use crate::starboard;

/// Whether the brush shape, moved along `t -> centre - (start + t * step)` for `t` in `[0, 1]`,
/// ever covers the point at offset `start` from the brush's first centre. Exact, not sampled.
#[must_use]
pub fn segment_meets_shape(
    shape: starboard::identity::Shape,
    radius: u16,
    pieces: &[Vec<starboard::canvas::HalfPlane>],
    start: (i64, i64),
    step: (i64, i64),
) -> bool {
    let sweep = starboard::canvas::SegmentSweep { start, step };
    if shape == starboard::identity::Shape::Circle {
        return sweep.meets_circle(i64::from(radius.max(1)));
    }
    pieces.iter().any(|piece| sweep.meets_piece(piece))
}
