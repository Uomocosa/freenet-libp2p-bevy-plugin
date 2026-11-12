use crate::starboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentSweep {
    pub start: (i64, i64),
    pub step: (i64, i64),
}

impl SegmentSweep {
    pub(crate) fn meets_circle(&self, radius: i64) -> bool {
        let (qx, qy) = (i128::from(self.start.0), i128::from(self.start.1));
        let (vx, vy) = (i128::from(self.step.0), i128::from(self.step.1));
        let r2 = i128::from(radius).saturating_mul(i128::from(radius));
        let len2 = vx.saturating_mul(vx).saturating_add(vy.saturating_mul(vy));
        let dot = qx.saturating_mul(vx).saturating_add(qy.saturating_mul(vy));
        let dist2 =
            |px: i128, py: i128| px.saturating_mul(px).saturating_add(py.saturating_mul(py));
        if len2 == 0 || dot <= 0 {
            return dist2(qx, qy) <= r2;
        }
        if dot >= len2 {
            return dist2(qx.saturating_sub(vx), qy.saturating_sub(vy)) <= r2;
        }
        let px = len2
            .saturating_mul(qx)
            .saturating_sub(dot.saturating_mul(vx));
        let py = len2
            .saturating_mul(qy)
            .saturating_sub(dot.saturating_mul(vy));
        dist2(px, py) <= len2.saturating_mul(len2).saturating_mul(r2)
    }

    pub(crate) fn meets_piece(&self, piece: &[starboard::canvas::HalfPlane]) -> bool {
        let mut low: (i128, i128) = (0, 1);
        let mut high: (i128, i128) = (1, 1);
        for half_plane in piece {
            let a = i128::from(half_plane.a);
            let b = i128::from(half_plane.b);
            let scale = i128::from(starboard::canvas::VERTEX_SCALE);
            let u = scale
                .saturating_mul(
                    a.saturating_mul(i128::from(self.start.0))
                        .saturating_add(b.saturating_mul(i128::from(self.start.1))),
                )
                .saturating_add(i128::from(half_plane.k));
            let w = scale.saturating_mul(
                a.saturating_mul(i128::from(self.step.0))
                    .saturating_add(b.saturating_mul(i128::from(self.step.1))),
            );
            match w.cmp(&0) {
                std::cmp::Ordering::Equal => {
                    if u < 0 {
                        return false;
                    }
                }
                std::cmp::Ordering::Greater => {
                    if Self::fraction_le((u, w), high) {
                        high = (u, w);
                    }
                }
                std::cmp::Ordering::Less => {
                    if Self::fraction_le(low, (u.saturating_neg(), w.saturating_neg())) {
                        low = (u.saturating_neg(), w.saturating_neg());
                    }
                }
            }
        }
        Self::fraction_le(low, high)
    }

    pub(crate) const fn fraction_le(left: (i128, i128), right: (i128, i128)) -> bool {
        left.0.saturating_mul(right.1) <= right.0.saturating_mul(left.1)
    }
}

#[cfg(test)]
mod tests {
    use super::SegmentSweep;

    #[test]
    fn test_usage() {
        let still = |x: i64| SegmentSweep {
            start: (x, 0),
            step: (0, 0),
        };
        assert!(still(0).meets_circle(1));
        assert!(!still(10).meets_circle(1));
        assert!(SegmentSweep::fraction_le((1, 2), (2, 3)));
    }
}
