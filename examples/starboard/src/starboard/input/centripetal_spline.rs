use bevy::prelude::{UVec2, Vec2};

use crate::starboard;

#[derive(Debug, Clone, Copy)]
pub struct CentripetalSpline {
    pub controls: [Vec2; 4],
    pub knots: [f32; 4],
}

impl CentripetalSpline {
    pub(crate) fn point_at(&self, knot: f32) -> Vec2 {
        let [before, start, end, after] = self.controls;
        let [knot_0, knot_1, knot_2, knot_3] = self.knots;
        let a_1 = Self::lerp_knot(before, start, knot_0, knot_1, knot);
        let a_2 = Self::lerp_knot(start, end, knot_1, knot_2, knot);
        let a_3 = Self::lerp_knot(end, after, knot_2, knot_3, knot);
        let b_1 = Self::lerp_knot(a_1, a_2, knot_0, knot_2, knot);
        let b_2 = Self::lerp_knot(a_2, a_3, knot_1, knot_3, knot);
        Self::lerp_knot(b_1, b_2, knot_1, knot_2, knot)
    }

    pub(crate) fn lerp_knot(
        first: Vec2,
        second: Vec2,
        knot_first: f32,
        knot_second: f32,
        knot: f32,
    ) -> Vec2 {
        let span = knot_second - knot_first;
        if span <= 0.0 {
            return first;
        }
        first.lerp(second, (knot - knot_first) / span)
    }

    pub(crate) fn to_vec2(point: starboard::canvas::SubCellPos) -> Vec2 {
        UVec2::new(point.x, point.y).as_vec2()
    }

    pub(crate) fn to_sub_cell(point: Vec2) -> starboard::canvas::SubCellPos {
        let rounded = point.round().as_uvec2();
        starboard::canvas::SubCellPos {
            x: rounded.x,
            y: rounded.y,
        }
    }

    pub(crate) fn push_distinct(
        points: &mut Vec<starboard::canvas::SubCellPos>,
        point: starboard::canvas::SubCellPos,
    ) {
        if points.last() != Some(&point) {
            points.push(point);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec2;

    use super::CentripetalSpline;

    #[test]
    fn test_usage() {
        let flat = CentripetalSpline {
            controls: [Vec2::ZERO, Vec2::X, Vec2::X * 2.0, Vec2::X * 3.0],
            knots: [0.0, 1.0, 2.0, 3.0],
        };
        assert_eq!(flat.point_at(1.5), Vec2::new(1.5, 0.0));
        assert_eq!(
            CentripetalSpline::lerp_knot(Vec2::ZERO, Vec2::X, 1.0, 1.0, 1.0),
            Vec2::ZERO
        );
    }
}
