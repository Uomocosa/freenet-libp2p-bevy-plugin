use bevy::prelude::*;

use crate::ui;

#[must_use]
pub fn roll_row_texts(app: &mut App, window: Entity) -> Option<[String; 2]> {
    let mut columns = app
        .world_mut()
        .query::<(Entity, &ui::roll::RollNumberColumn, &ChildOf)>();
    let column = columns
        .iter(app.world())
        .find(|(_, _, parent)| parent.parent() == window)
        .map(|(entity, _, _)| entity)?;
    let mut rows = app
        .world_mut()
        .query::<(&ui::roll::RollNumberRow, &ChildOf, &Text)>();
    let mut outgoing = String::new();
    let mut incoming = String::new();
    for (row, parent, text) in rows.iter(app.world()) {
        if parent.parent() != column {
            continue;
        }
        match row {
            ui::roll::RollNumberRow::Outgoing => outgoing = text.as_str().to_string(),
            ui::roll::RollNumberRow::Incoming => incoming = text.as_str().to_string(),
        }
    }
    Some([outgoing, incoming])
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;

    use crate::testing;
    use crate::ui;

    use super::roll_row_texts;

    #[test]
    fn test_usage() {
        let style = ui::roll::NumberStyle {
            font_size: 11.0,
            row_height: 10.0,
            text_color: Color::WHITE,
        };
        let mut app = testing::roll_app_with_numbers(&[9, 9], Duration::from_millis(50), style);
        let window = testing::roll_number_entity(&mut app, 0).expect("the number window");
        assert_eq!(
            roll_row_texts(&mut app, window),
            Some([String::from("9"), String::from("9")])
        );
    }
}
