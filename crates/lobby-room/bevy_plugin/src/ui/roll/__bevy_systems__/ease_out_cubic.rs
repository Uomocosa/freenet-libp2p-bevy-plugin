#[must_use]
pub fn ease_out_cubic(progress: f32) -> f32 {
    let inverted = 1.0 - progress.clamp(0.0, 1.0);
    (inverted * inverted).mul_add(-inverted, 1.0)
}

#[cfg(test)]
mod tests {
    use super::ease_out_cubic;

    #[test]
    fn test_usage() {
        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
    }

    #[test]
    fn the_curve_runs_ahead_of_the_straight_line() {
        assert!(ease_out_cubic(0.5) > 0.5);
        assert!(ease_out_cubic(0.25) > 0.25);
    }

    #[test]
    fn a_progress_outside_the_unit_interval_is_clamped() {
        assert_eq!(ease_out_cubic(-0.5), 0.0);
        assert_eq!(ease_out_cubic(1.5), 1.0);
    }
}
