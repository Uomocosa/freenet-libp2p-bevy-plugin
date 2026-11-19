use bevy::prelude::*;
use bevy::ui::UiSystems;

use crate::ui;

pub struct UiPlugin;

impl bevy::prelude::Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ui::roll::bevy_systems::drive_roll_machines.in_set(ui::roll::RollSet::Drive),
        )
        .add_systems(
            PostUpdate,
            ui::roll::bevy_systems::resize_roll_numbers.after(UiSystems::Layout),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::ui::ComputedNode;

    use super::UiPlugin;
    use crate::testing;
    use crate::ui;
    use ui::roll::NumberStyle;

    const ROW_HEIGHT: f32 = 10.0;
    const FRAME_MILLIS: u64 = 50;

    const STYLE: NumberStyle = NumberStyle {
        font_size: 11.0,
        row_height: ROW_HEIGHT,
        text_color: Color::WHITE,
    };

    const LAYOUT_STYLE: NumberStyle = NumberStyle {
        font_size: 11.0,
        row_height: 13.0,
        text_color: Color::WHITE,
    };

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(UiPlugin);
        app.update();
        assert!(app.world().resource::<Time>().delta_secs() >= 0.0);
    }

    #[test]
    fn a_roll_slides_the_incoming_number_up_to_the_settled_value() {
        let mut app =
            testing::roll_app_with_numbers(&[9, 9], Duration::from_millis(FRAME_MILLIS), STYLE);
        let friends = testing::roll_number_entity(&mut app, 1).expect("the number window");
        testing::show_roll_numbers(&mut app, &[9, 10]);

        app.update();
        let offset = testing::roll_column_offset(&mut app, friends).expect("the column offset");
        assert!(
            (offset - -4.212_963).abs() < 0.05,
            "expected -4.212963, got {offset}"
        );
        assert_eq!(
            testing::roll_row_texts(&mut app, friends),
            Some([String::from("9"), String::from("10")])
        );
        assert_eq!(
            testing::roll_number_value(&mut app, 1),
            Some(9),
            "mid-roll the value is stale"
        );

        app.update();
        app.update();
        let offset = testing::roll_column_offset(&mut app, friends).expect("the column offset");
        assert!(
            (offset - -8.75).abs() < 0.05,
            "expected -8.75, got {offset}"
        );

        for _ in 0..3 {
            app.update();
        }
        assert_eq!(testing::roll_column_offset(&mut app, friends), Some(0.0));
        assert_eq!(
            testing::roll_row_texts(&mut app, friends),
            Some([String::from("10"), String::from("10")])
        );
        assert_eq!(testing::roll_number_value(&mut app, 1), Some(10));
        let mut machines = app.world_mut().query::<&ui::roll::RollMachine>();
        let machine = machines
            .iter(app.world())
            .next()
            .map(|machine| ui::roll::State::clone(machine))
            .expect("the machine");
        assert!(matches!(machine.phase, ui::roll::Phase::Idle));
    }

    #[test]
    fn a_single_number_can_roll_down_to_zero() {
        let mut app =
            testing::roll_app_with_numbers(&[9, 9], Duration::from_millis(FRAME_MILLIS), STYLE);
        let occupants = testing::roll_number_entity(&mut app, 0).expect("the number window");
        testing::show_roll_numbers(&mut app, &[0, 9]);

        for _ in 0..6 {
            app.update();
        }
        assert_eq!(testing::roll_column_offset(&mut app, occupants), Some(0.0));
        assert_eq!(
            testing::roll_row_texts(&mut app, occupants),
            Some([String::from("0"), String::from("0")])
        );
        assert_eq!(testing::roll_number_value(&mut app, 0), Some(0));
    }

    #[test]
    fn a_number_window_takes_the_width_of_the_rows_it_has_measured() {
        for (outgoing_width, incoming_width, expected_width) in [
            (6.0, 12.0, Val::Px(12.0)),
            (6.0, 0.0, Val::Px(6.0)),
            (0.0, 12.0, Val::Px(12.0)),
            (0.0, 0.0, Val::Auto),
        ] {
            let mut app =
                testing::roll_app_with_numbers(&[9], Duration::from_millis(FRAME_MILLIS), STYLE);
            let mut row_query = app
                .world_mut()
                .query::<(Entity, &ui::roll::RollNumberRow)>();
            let rows: Vec<(Entity, ui::roll::RollNumberRow)> = row_query
                .iter(app.world())
                .map(|(entity, row)| (entity, *row))
                .collect();
            assert_eq!(rows.len(), 2, "one outgoing and one incoming row");
            for (entity, row) in rows {
                let width = match row {
                    ui::roll::RollNumberRow::Outgoing => outgoing_width,
                    ui::roll::RollNumberRow::Incoming => incoming_width,
                };
                app.world_mut().entity_mut(entity).insert(ComputedNode {
                    size: Vec2::new(width, ROW_HEIGHT),
                    ..default()
                });
            }
            app.update();
            let mut windows = app
                .world_mut()
                .query_filtered::<&Node, With<ui::roll::RollNumber>>();
            let window = windows.single(app.world()).expect("one number window");
            assert_eq!(
                window.width, expected_width,
                "outgoing {outgoing_width}, incoming {incoming_width}"
            );
        }
    }

    #[test]
    fn a_static_text_and_a_rolled_number_share_the_same_line_box() {
        let mut app = testing::roll_app_with_layout(LAYOUT_STYLE);
        let mut static_nodes = app
            .world_mut()
            .query_filtered::<&ComputedNode, (With<Text>, Without<ui::roll::RollNumberRow>)>();
        let static_height = static_nodes
            .single(app.world())
            .expect("the static text")
            .size()
            .y;
        let mut rolled_nodes = app
            .world_mut()
            .query::<(&ui::roll::RollNumberRow, &ComputedNode)>();
        let heights: Vec<f32> = rolled_nodes
            .iter(app.world())
            .map(|(_, node)| node.size().y)
            .collect();
        assert_eq!(heights.len(), 2);
        for height in heights {
            assert!(
                (height - static_height).abs() < 0.01,
                "rolled {height} vs static {static_height}"
            );
            assert!(
                (height - LAYOUT_STYLE.row_height).abs() < 0.01,
                "line box {height}"
            );
        }
    }

    #[test]
    fn nine_to_ten_grows_the_window_without_a_single_frame_jump() {
        let mut app = testing::roll_app_with_layout(LAYOUT_STYLE);
        let mut widths = vec![testing::roll_window_width(&mut app).expect("the window width")];
        testing::show_roll_numbers(&mut app, &[10]);
        for _ in 0..10 {
            app.update();
            widths.push(testing::roll_window_width(&mut app).expect("the window width"));
        }
        let steps: Vec<f32> = widths.windows(2).map(|pair| pair[1] - pair[0]).collect();
        assert!(
            steps.iter().all(|step| *step >= -0.01),
            "width is monotonic: {widths:?}"
        );
        let largest = steps.iter().copied().fold(0.0_f32, f32::max);
        assert!(largest <= 1.0, "largest step {largest} in {widths:?}");
        let first = widths.first().copied().unwrap_or_default();
        let last = widths.last().copied().unwrap_or_default();
        assert!(last > first, "the window grew from {first} to {last}");
    }
}
