use bevy::prelude::{UVec2, Vec2};

use crate::starboard;

/// Distance between flattened samples, in sub-cell units (a quarter cell).
const SAMPLE_SPACING_SUBCELLS: f32 = 2.0;

/// The centripetal Catmull-Rom segment from `controls[1]` to `controls[2]`, flattened into
/// sub-cell points that start and end exactly on those two controls.
#[must_use]
pub fn catmull_rom_points(
    controls: [starboard::canvas::SubCellPos; 4],
) -> Vec<starboard::canvas::SubCellPos> {
    let [before, start, end, after] = controls.map(to_vec2);
    let knot_1 = start.distance(before).sqrt();
    let knot_2 = knot_1 + end.distance(start).sqrt();
    let knot_3 = knot_2 + after.distance(end).sqrt();
    let knots = [0.0, knot_1, knot_2, knot_3];
    let chord = end.distance(start);
    let mut points = vec![to_sub_cell(start)];
    if chord > 0.0 {
        let fraction_step = SAMPLE_SPACING_SUBCELLS / chord;
        let fractions = std::iter::successors(Some(fraction_step), |fraction| {
            Some(fraction + fraction_step)
        })
        .take_while(|fraction| *fraction < 1.0);
        for fraction in fractions {
            let knot = (knot_2 - knot_1).mul_add(fraction, knot_1);
            let point = curve_point([before, start, end, after], knots, knot);
            push_distinct(&mut points, to_sub_cell(point));
        }
    }
    push_distinct(&mut points, to_sub_cell(end));
    points
}

// needed helper: one point of the spline at knot `knot`, by the Barry-Goldman pyramid
fn curve_point(controls: [Vec2; 4], knots: [f32; 4], knot: f32) -> Vec2 {
    let [before, start, end, after] = controls;
    let [knot_0, knot_1, knot_2, knot_3] = knots;
    let a_1 = lerp_knot(before, start, knot_0, knot_1, knot);
    let a_2 = lerp_knot(start, end, knot_1, knot_2, knot);
    let a_3 = lerp_knot(end, after, knot_2, knot_3, knot);
    let b_1 = lerp_knot(a_1, a_2, knot_0, knot_2, knot);
    let b_2 = lerp_knot(a_2, a_3, knot_1, knot_3, knot);
    lerp_knot(b_1, b_2, knot_1, knot_2, knot)
}

// needed helper: a zero knot interval means the two points are equal, so the first one is the answer
fn lerp_knot(first: Vec2, second: Vec2, knot_first: f32, knot_second: f32, knot: f32) -> Vec2 {
    let span = knot_second - knot_first;
    if span <= 0.0 {
        return first;
    }
    first.lerp(second, (knot - knot_first) / span)
}

// needed helper: a sub-cell point as a float vector, exact for every canvas coordinate
fn to_vec2(point: starboard::canvas::SubCellPos) -> Vec2 {
    UVec2::new(point.x, point.y).as_vec2()
}

// needed helper: the nearest sub-cell point; negative overshoot saturates at the canvas edge
fn to_sub_cell(point: Vec2) -> starboard::canvas::SubCellPos {
    let rounded = point.round().as_uvec2();
    starboard::canvas::SubCellPos {
        x: rounded.x,
        y: rounded.y,
    }
}

// needed helper: append unless the point repeats the last one, so no zero-length sample is sent
fn push_distinct(
    points: &mut Vec<starboard::canvas::SubCellPos>,
    point: starboard::canvas::SubCellPos,
) {
    if points.last() != Some(&point) {
        points.push(point);
    }
}

#[cfg(test)]
mod tests {
    use super::catmull_rom_points;
    use crate::starboard::canvas::SubCellPos;

    fn at(x: u32, y: u32) -> SubCellPos {
        SubCellPos { x, y }
    }

    #[test]
    fn test_usage() {
        let points = catmull_rom_points([at(0, 0), at(0, 0), at(80, 0), at(80, 0)]);
        assert_eq!(points.first(), Some(&at(0, 0)));
        assert_eq!(points.last(), Some(&at(80, 0)));
        assert!(points.iter().all(|point| point.y == 0 && point.x <= 80));
        assert!(points.windows(2).all(|pair| {
            pair.first()
                .zip(pair.last())
                .is_some_and(|(left, right)| left.x < right.x)
        }));
    }

    #[test]
    fn a_zero_length_segment_is_one_point() {
        assert_eq!(
            catmull_rom_points([at(5, 5), at(5, 5), at(5, 5), at(5, 5)]),
            vec![at(5, 5)]
        );
    }

    #[test]
    fn the_first_and_last_segments_of_a_curve_stay_on_the_curve() {
        let curve = [at(0, 0), at(0, 0), at(40, 60), at(120, 40)];
        let points = catmull_rom_points(curve);
        assert_eq!(points.first(), Some(&at(0, 0)));
        assert_eq!(points.last(), Some(&at(40, 60)));
        assert!(points.iter().all(|point| point.x < 200 && point.y < 200));
    }

    #[test]
    fn neighbouring_samples_are_at_most_three_sub_cells_apart_after_rounding() {
        let curve = [at(100, 100), at(200, 140), at(300, 100), at(400, 180)];
        let points = catmull_rom_points(curve);
        assert!(points.windows(2).all(|pair| {
            pair.first().zip(pair.last()).is_some_and(|(left, right)| {
                let dx = i64::from(left.x).abs_diff(i64::from(right.x));
                let dy = i64::from(left.y).abs_diff(i64::from(right.y));
                dx * dx + dy * dy <= 9
            })
        }));
    }
}
