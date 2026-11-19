use bevy::prelude::{App, Entity};

use crate::ui;

#[must_use]
pub fn roll_number_entity(app: &mut App, slot: usize) -> Option<Entity> {
    let mut numbers = app.world_mut().query::<(Entity, &ui::roll::RollNumber)>();
    numbers
        .iter(app.world())
        .find(|(_, number)| number.slot == slot)
        .map(|(entity, _)| entity)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;

    use crate::testing;
    use crate::ui;

    use super::roll_number_entity;

    #[test]
    fn test_usage() {
        let style = ui::roll::NumberStyle {
            font_size: 11.0,
            row_height: 10.0,
            text_color: Color::WHITE,
        };
        let mut app = testing::roll_app_with_numbers(&[9, 9], Duration::from_millis(50), style);
        assert!(roll_number_entity(&mut app, 0).is_some());
        assert!(roll_number_entity(&mut app, 2).is_none());
    }
}
