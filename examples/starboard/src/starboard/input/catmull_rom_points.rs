use crate::starboard;

const SAMPLE_SPACING_SUBCELLS: f32 = 2.0;

#[must_use]
pub fn catmull_rom_points(
    controls: [starboard::canvas::SubCellPos; 4],
) -> Vec<starboard::canvas::SubCellPos> {
    let [before, start, end, after] = controls.map(starboard::input::CentripetalSpline::to_vec2);
    let knot_1 = start.distance(before).sqrt();
    let knot_2 = knot_1 + end.distance(start).sqrt();
    let knot_3 = knot_2 + after.distance(end).sqrt();
    let knots = [0.0, knot_1, knot_2, knot_3];
    let spline = starboard::input::CentripetalSpline {
        controls: [before, start, end, after],
        knots,
    };
    let chord = end.distance(start);
    let mut points = vec![starboard::input::CentripetalSpline::to_sub_cell(start)];
    if chord > 0.0 {
        let fraction_step = SAMPLE_SPACING_SUBCELLS / chord;
        let fractions = std::iter::successors(Some(fraction_step), |fraction| {
            Some(fraction + fraction_step)
        })
        .take_while(|fraction| *fraction < 1.0);
        for fraction in fractions {
            let knot = (knot_2 - knot_1).mul_add(fraction, knot_1);
            let point = spline.point_at(knot);
            starboard::input::CentripetalSpline::push_distinct(
                &mut points,
                starboard::input::CentripetalSpline::to_sub_cell(point),
            );
        }
    }
    starboard::input::CentripetalSpline::push_distinct(
        &mut points,
        starboard::input::CentripetalSpline::to_sub_cell(end),
    );
    points
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
