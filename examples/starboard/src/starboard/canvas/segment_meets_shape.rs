use crate::starboard;
use crate::starboard::canvas::HalfPlane;

const VERTEX_SCALE: i64 = 1000;

/// Whether the brush shape, moved along `t -> centre - (start + t * step)` for `t` in `[0, 1]`,
/// ever covers the point at offset `start` from the brush's first centre. Exact, not sampled.
#[must_use]
pub fn segment_meets_shape(
    shape: starboard::identity::Shape,
    radius: u16,
    pieces: &[Vec<HalfPlane>],
    start: (i64, i64),
    step: (i64, i64),
) -> bool {
    if shape == starboard::identity::Shape::Circle {
        return circle_meets(i64::from(radius.max(1)), start, step);
    }
    pieces.iter().any(|piece| piece_meets(piece, start, step))
}

// needed helper: the squared distance from a point to a moving centre is a quadratic in t, so its
// minimum over [0, 1] is at the projection of the point onto the segment, clamped to the ends
fn circle_meets(radius: i64, start: (i64, i64), step: (i64, i64)) -> bool {
    let (qx, qy) = (i128::from(start.0), i128::from(start.1));
    let (vx, vy) = (i128::from(step.0), i128::from(step.1));
    let r2 = i128::from(radius).saturating_mul(i128::from(radius));
    let len2 = vx.saturating_mul(vx).saturating_add(vy.saturating_mul(vy));
    let dot = qx.saturating_mul(vx).saturating_add(qy.saturating_mul(vy));
    let dist2 = |px: i128, py: i128| px.saturating_mul(px).saturating_add(py.saturating_mul(py));
    if len2 == 0 || dot <= 0 {
        return dist2(qx, qy) <= r2;
    }
    if dot >= len2 {
        return dist2(qx.saturating_sub(vx), qy.saturating_sub(vy)) <= r2;
    }
    // at t = dot / len2: |Q - t V|^2 <= r^2 becomes |len2 Q - dot V|^2 <= len2^2 r^2
    let px = len2
        .saturating_mul(qx)
        .saturating_sub(dot.saturating_mul(vx));
    let py = len2
        .saturating_mul(qy)
        .saturating_sub(dot.saturating_mul(vy));
    dist2(px, py) <= len2.saturating_mul(len2).saturating_mul(r2)
}

// needed helper: each half-plane gives `u - t * w >= 0` along the segment; the piece is hit when
// the `t` ranges of all its half-planes overlap inside [0, 1]
fn piece_meets(piece: &[HalfPlane], start: (i64, i64), step: (i64, i64)) -> bool {
    let mut low: (i128, i128) = (0, 1);
    let mut high: (i128, i128) = (1, 1);
    for half_plane in piece {
        let a = i128::from(half_plane.a);
        let b = i128::from(half_plane.b);
        let scale = i128::from(VERTEX_SCALE);
        let u = scale
            .saturating_mul(
                a.saturating_mul(i128::from(start.0))
                    .saturating_add(b.saturating_mul(i128::from(start.1))),
            )
            .saturating_add(i128::from(half_plane.k));
        let w = scale.saturating_mul(
            a.saturating_mul(i128::from(step.0))
                .saturating_add(b.saturating_mul(i128::from(step.1))),
        );
        match w.cmp(&0) {
            std::cmp::Ordering::Equal => {
                if u < 0 {
                    return false;
                }
            }
            std::cmp::Ordering::Greater => {
                if fraction_le((u, w), high) {
                    high = (u, w);
                }
            }
            std::cmp::Ordering::Less => {
                if fraction_le(low, (u.saturating_neg(), w.saturating_neg())) {
                    low = (u.saturating_neg(), w.saturating_neg());
                }
            }
        }
    }
    fraction_le(low, high)
}

// needed helper: compares two fractions whose denominators are positive, without dividing
const fn fraction_le(left: (i128, i128), right: (i128, i128)) -> bool {
    left.0.saturating_mul(right.1) <= right.0.saturating_mul(left.1)
}
