use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use crate::ui;

pub fn roll_app_with_numbers(
    numbers: &[u32],
    frame: Duration,
    style: ui::roll::NumberStyle,
) -> App {
    let numbers = numbers.to_vec();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(frame));
    app.add_systems(Startup, move |mut commands: Commands| {
        let pill = commands
            .spawn(ui::roll::RollMachine(ui::roll::State::new(
                numbers.clone(),
                None,
            )))
            .id();
        commands.entity(pill).with_children(|machine| {
            for (slot, number) in numbers.iter().copied().enumerate() {
                ui::roll::bevy_systems::spawn_roll_number(
                    &mut machine.reborrow(),
                    pill,
                    slot,
                    number,
                    style,
                );
            }
        });
    });
    app.add_plugins(ui::roll::UiPlugin);
    app.update();
    app
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;

    use crate::ui;

    use super::roll_app_with_numbers;

    #[test]
    fn test_usage() {
        let style = ui::roll::NumberStyle {
            font_size: 11.0,
            row_height: 10.0,
            text_color: Color::WHITE,
        };
        let app = roll_app_with_numbers(&[9], Duration::from_millis(50), style);
        assert!(app.world().get_resource::<Time>().is_some());
    }
}
