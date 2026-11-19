use bevy::prelude::App;

use crate::ui;

pub fn show_roll_numbers(app: &mut App, numbers: &[u32]) {
    let mut machines = app.world_mut().query::<&mut ui::roll::RollMachine>();
    for mut machine in machines.iter_mut(app.world_mut()) {
        let _ = machine.update(ui::roll::Input::Show {
            numbers: numbers.to_vec(),
        });
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;

    use crate::testing;
    use crate::ui;

    use super::show_roll_numbers;

    #[test]
    fn test_usage() {
        let style = ui::roll::NumberStyle {
            font_size: 11.0,
            row_height: 10.0,
            text_color: Color::WHITE,
        };
        let mut app = testing::roll_app_with_numbers(&[9, 9], Duration::from_millis(50), style);
        show_roll_numbers(&mut app, &[9, 10]);
        assert_eq!(testing::roll_number_value(&mut app, 1), Some(9));
    }
}
