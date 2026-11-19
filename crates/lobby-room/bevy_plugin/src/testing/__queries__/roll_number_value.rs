use bevy::prelude::App;

use crate::ui;

#[must_use]
pub fn roll_number_value(app: &mut App, slot: usize) -> Option<u32> {
    let mut numbers = app.world_mut().query::<&ui::roll::RollNumber>();
    numbers
        .iter(app.world())
        .find(|number| number.slot == slot)
        .map(|number| number.value)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;

    use crate::testing;
    use crate::ui;

    use super::roll_number_value;

    #[test]
    fn test_usage() {
        let style = ui::roll::NumberStyle {
            font_size: 11.0,
            row_height: 10.0,
            text_color: Color::WHITE,
        };
        let mut app = testing::roll_app_with_numbers(&[9, 9], Duration::from_millis(50), style);
        assert_eq!(roll_number_value(&mut app, 0), Some(9));
        assert_eq!(roll_number_value(&mut app, 2), None);
    }
}
