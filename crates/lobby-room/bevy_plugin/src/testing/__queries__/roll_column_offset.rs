use bevy::prelude::*;

use crate::ui;

#[must_use]
pub fn roll_column_offset(app: &mut App, window: Entity) -> Option<f32> {
    let mut columns = app
        .world_mut()
        .query::<(Entity, &ui::roll::RollNumberColumn, &ChildOf)>();
    let column = columns
        .iter(app.world())
        .find(|(_, _, parent)| parent.parent() == window)
        .map(|(entity, _, _)| entity)?;
    let mut transforms = app.world_mut().query::<&UiTransform>();
    let transform = transforms.get(app.world(), column).ok()?;
    let Val::Px(offset) = transform.translation.y else {
        return None;
    };
    Some(offset)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;

    use crate::testing;
    use crate::ui;

    use super::roll_column_offset;

    #[test]
    fn test_usage() {
        let style = ui::roll::NumberStyle {
            font_size: 11.0,
            row_height: 10.0,
            text_color: Color::WHITE,
        };
        let mut app = testing::roll_app_with_numbers(&[9, 9], Duration::from_millis(50), style);
        let window = testing::roll_number_entity(&mut app, 0).expect("the number window");
        assert_eq!(roll_column_offset(&mut app, window), Some(0.0));
    }
}
