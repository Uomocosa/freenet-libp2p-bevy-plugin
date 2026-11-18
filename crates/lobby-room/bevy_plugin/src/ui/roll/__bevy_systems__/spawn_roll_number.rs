use bevy::prelude::*;

use crate::ui;

pub fn spawn_roll_number(
    parent: &mut ChildSpawnerCommands,
    pill: Entity,
    slot: usize,
    value: u32,
    style: ui::roll::NumberStyle,
) -> Entity {
    let glyphs = value.to_string();
    let mut window = parent.spawn((
        ui::roll::RollNumber {
            pill,
            slot,
            value,
            row_height: style.row_height,
        },
        Node {
            width: Val::Auto,
            height: Val::Px(style.row_height),
            overflow: Overflow::clip_y(),
            flex_shrink: 0.0,
            ..default()
        },
    ));
    let window_id = window.id();
    window.with_children(|columns| {
        columns
            .spawn((
                ui::roll::RollNumberColumn,
                UiTransform::default(),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(2.0 * style.row_height),
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.0,
                    align_items: AlignItems::FlexEnd,
                    ..default()
                },
            ))
            .with_children(|column| {
                column.spawn((
                    ui::roll::RollNumberRow::Outgoing,
                    Text::new(glyphs.clone()),
                    style.text_font(),
                    style.line_height(),
                    TextColor(style.text_color),
                    TextLayout::no_wrap(),
                ));
                column.spawn((
                    ui::roll::RollNumberRow::Incoming,
                    Text::new(glyphs),
                    style.text_font(),
                    style.line_height(),
                    TextColor(style.text_color),
                    TextLayout::no_wrap(),
                ));
            });
    });
    window_id
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::text::LineHeight;

    use super::spawn_roll_number;
    use crate::ui;
    use ui::roll::NumberStyle;

    const STYLE: NumberStyle = NumberStyle {
        font_size: 11.0,
        row_height: 13.0,
        text_color: Color::WHITE,
    };

    fn app_with(value: u32) -> App {
        let mut app = App::new();
        app.add_systems(Startup, move |mut commands: Commands| {
            let pill = commands.spawn_empty().id();
            commands.entity(pill).with_children(|parent| {
                spawn_roll_number(&mut parent.reborrow(), pill, 0, value, STYLE);
            });
        });
        app.update();
        app
    }

    fn rows(app: &mut App) -> Vec<(ui::roll::RollNumberRow, String, LineHeight)> {
        let mut query = app
            .world_mut()
            .query::<(&ui::roll::RollNumberRow, &Text, &LineHeight)>();
        let mut found: Vec<(ui::roll::RollNumberRow, String, LineHeight)> = query
            .iter(app.world())
            .map(|(row, text, line_height)| (*row, text.as_str().to_string(), *line_height))
            .collect();
        found.sort_by_key(|(row, _, _)| match row {
            ui::roll::RollNumberRow::Outgoing => 0,
            ui::roll::RollNumberRow::Incoming => 1,
        });
        found
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(42);
        let mut numbers = app
            .world_mut()
            .query::<(&ui::roll::RollNumber, &Children, &Node)>();
        let (number, columns, node) = numbers.single(app.world()).expect("one number");
        assert_eq!(number.value, 42);
        assert_eq!(node.height, Val::Px(13.0));
        let column = columns.iter().next().expect("one column");
        let column_children = app
            .world()
            .get::<Children>(column)
            .expect("the column children");
        assert_eq!(column_children.len(), 2, "exactly two whole-number texts");
        let rows = rows(&mut app);
        assert_eq!(
            rows,
            vec![
                (
                    ui::roll::RollNumberRow::Outgoing,
                    String::from("42"),
                    LineHeight::Px(13.0)
                ),
                (
                    ui::roll::RollNumberRow::Incoming,
                    String::from("42"),
                    LineHeight::Px(13.0)
                ),
            ]
        );
    }

    #[test]
    fn a_spawned_number_starts_with_its_column_at_the_top() {
        let mut app = app_with(9);
        let mut columns = app
            .world_mut()
            .query::<(&ui::roll::RollNumberColumn, &UiTransform)>();
        let (_, transform) = columns.single(app.world()).expect("one column");
        assert_eq!(transform.translation, Val2::px(0.0, 0.0));
    }
}
