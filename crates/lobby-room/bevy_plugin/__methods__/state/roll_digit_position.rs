use crate::ui;

pub fn roll_digit_position(from: u32, to: u32, place: usize, progress: f32) -> f32 {
    let start_digit = ui::roll::State::digit_at(from, place);
    let end_digit = ui::roll::State::digit_at(to, place);
    let rolled_end = if end_digit < start_digit {
        end_digit.saturating_add(10)
    } else {
        end_digit
    };
    (f32::from(rolled_end) - f32::from(start_digit)).mul_add(progress, f32::from(start_digit))
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use ui::roll::State;

    #[test]
    fn test_usage() {
        assert_eq!(State::roll_digit_position(9, 0, 0, 0.0), 9.0);
    }

    #[test]
    fn ten_to_eleven_rolls_both_digits() {
        assert_eq!(State::roll_digit_position(10, 11, 0, 0.5), 0.5);
        assert_eq!(State::roll_digit_position(10, 11, 1, 0.5), 1.0);
        assert_eq!(State::roll_digit_position(9, 10, 0, 0.5), 9.5);
        assert_eq!(State::roll_digit_position(9, 10, 1, 0.5), 0.5);
    }
}
