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
    use bevy::time::TimeUpdateStrategy;
    use bevy::ui::ComputedNode;

    use super::UiPlugin;
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

    fn app_with_two_numbers() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            FRAME_MILLIS,
        )));
        app.add_systems(Startup, |mut commands: Commands| {
            let pill = commands
                .spawn(ui::roll::RollMachine(ui::roll::State::new(
                    vec![9, 9],
                    None,
                )))
                .id();
            commands.entity(pill).with_children(|machine| {
                ui::roll::bevy_systems::spawn_roll_number(
                    &mut machine.reborrow(),
                    pill,
                    0,
                    9,
                    STYLE,
                );
                ui::roll::bevy_systems::spawn_roll_number(
                    &mut machine.reborrow(),
                    pill,
                    1,
                    9,
                    STYLE,
                );
            });
        });
        app.add_plugins(UiPlugin);
        app.update();
        app
    }

    fn show(app: &mut App, numbers: &[u32]) {
        let mut machines = app.world_mut().query::<&mut ui::roll::RollMachine>();
        for mut machine in machines.iter_mut(app.world_mut()) {
            let _ = machine.update(ui::roll::Input::Show {
                numbers: numbers.to_vec(),
            });
        }
    }

    fn number_entity(app: &mut App, slot: usize) -> Entity {
        let mut numbers = app.world_mut().query::<(Entity, &ui::roll::RollNumber)>();
        numbers
            .iter(app.world())
            .find(|(_, number)| number.slot == slot)
            .map(|(entity, _)| entity)
            .expect("the number window")
    }

    fn number_value(app: &mut App, slot: usize) -> u32 {
        let mut numbers = app.world_mut().query::<&ui::roll::RollNumber>();
        numbers
            .iter(app.world())
            .find(|number| number.slot == slot)
            .map(|number| number.value)
            .expect("the number window")
    }

    fn column_offset(app: &mut App, window: Entity) -> f32 {
        let mut columns = app
            .world_mut()
            .query::<(Entity, &ui::roll::RollNumberColumn, &ChildOf)>();
        let column = columns
            .iter(app.world())
            .find(|(_, _, parent)| parent.parent() == window)
            .map(|(entity, _, _)| entity)
            .expect("the number column");
        let mut transforms = app.world_mut().query::<&UiTransform>();
        let transform = transforms
            .get(app.world(), column)
            .expect("the number column");
        let Val::Px(offset) = transform.translation.y else {
            panic!("the column translates in pixels")
        };
        offset
    }

    fn row_texts(app: &mut App, window: Entity) -> [String; 2] {
        let mut columns = app
            .world_mut()
            .query::<(Entity, &ui::roll::RollNumberColumn, &ChildOf)>();
        let column = columns
            .iter(app.world())
            .find(|(_, _, parent)| parent.parent() == window)
            .map(|(entity, _, _)| entity)
            .expect("the number column");
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
        [outgoing, incoming]
    }

    fn assert_offset(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.05,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn a_roll_slides_the_incoming_number_up_to_the_settled_value() {
        let mut app = app_with_two_numbers();
        let friends = number_entity(&mut app, 1);
        show(&mut app, &[9, 10]);

        app.update();
        assert_offset(column_offset(&mut app, friends), -4.212_963);
        assert_eq!(
            row_texts(&mut app, friends),
            [String::from("9"), String::from("10")]
        );
        assert_eq!(number_value(&mut app, 1), 9, "mid-roll the value is stale");

        app.update();
        app.update();
        assert_offset(column_offset(&mut app, friends), -8.75);

        for _ in 0..3 {
            app.update();
        }
        assert_eq!(column_offset(&mut app, friends), 0.0);
        assert_eq!(
            row_texts(&mut app, friends),
            [String::from("10"), String::from("10")]
        );
        assert_eq!(number_value(&mut app, 1), 10);
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
        let mut app = app_with_two_numbers();
        let occupants = number_entity(&mut app, 0);
        show(&mut app, &[0, 9]);

        for _ in 0..6 {
            app.update();
        }
        assert_eq!(column_offset(&mut app, occupants), 0.0);
        assert_eq!(
            row_texts(&mut app, occupants),
            [String::from("0"), String::from("0")]
        );
        assert_eq!(number_value(&mut app, 0), 0);
    }

    #[test]
    fn a_number_window_takes_the_width_of_the_rows_it_has_measured() {
        for (outgoing_width, incoming_width, expected_width) in [
            (6.0, 12.0, Val::Px(12.0)),
            (6.0, 0.0, Val::Px(6.0)),
            (0.0, 12.0, Val::Px(12.0)),
            (0.0, 0.0, Val::Auto),
        ] {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_systems(Startup, |mut commands: Commands| {
                let pill = commands
                    .spawn(ui::roll::RollMachine(ui::roll::State::new(vec![9], None)))
                    .id();
                commands.entity(pill).with_children(|parent| {
                    ui::roll::bevy_systems::spawn_roll_number(
                        &mut parent.reborrow(),
                        pill,
                        0,
                        9,
                        STYLE,
                    );
                });
            });
            app.add_plugins(UiPlugin);
            app.update();
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

    fn app_with_layout() -> App {
        let mut app = App::new();
        app.add_plugins(
            DefaultPlugins
                .build()
                .disable::<bevy::winit::WinitPlugin>()
                .disable::<bevy::window::WindowPlugin>()
                .disable::<bevy::a11y::AccessibilityPlugin>(),
        );
        app.add_message::<bevy::window::WindowEvent>()
            .add_message::<bevy::window::WindowResized>()
            .add_message::<bevy::window::WindowCreated>()
            .add_message::<bevy::window::WindowClosing>()
            .add_message::<bevy::window::WindowClosed>()
            .add_message::<bevy::window::WindowCloseRequested>()
            .add_message::<bevy::window::WindowDestroyed>()
            .add_message::<bevy::window::WindowFocused>()
            .add_message::<bevy::window::WindowOccluded>()
            .add_message::<bevy::window::WindowScaleFactorChanged>()
            .add_message::<bevy::window::WindowBackendScaleFactorChanged>()
            .add_message::<bevy::window::WindowMoved>()
            .add_message::<bevy::window::WindowThemeChanged>()
            .add_message::<bevy::window::Ime>();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_nanos(
            33_333_333,
        )));
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                Camera2d,
                Camera {
                    computed: bevy::camera::ComputedCameraValues {
                        target_info: Some(bevy::camera::RenderTargetInfo {
                            physical_size: UVec2::new(800, 600),
                            scale_factor: 1.0,
                        }),
                        ..default()
                    },
                    ..default()
                },
                bevy::ui::IsDefaultUiCamera,
            ));
            let root = commands
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .id();
            commands.spawn((
                ChildOf(root),
                Text::new("8"),
                LAYOUT_STYLE.text_font(),
                LAYOUT_STYLE.line_height(),
                TextColor(Color::WHITE),
                TextLayout::no_wrap(),
            ));
            let pill = commands
                .spawn((
                    ChildOf(root),
                    ui::roll::RollMachine(ui::roll::State::new(vec![9], None)),
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .id();
            commands.entity(pill).with_children(|parent| {
                ui::roll::bevy_systems::spawn_roll_number(
                    &mut parent.reborrow(),
                    pill,
                    0,
                    9,
                    LAYOUT_STYLE,
                );
            });
        });
        app.add_plugins(UiPlugin);
        app.finish();
        app.cleanup();
        app.update();
        app
    }

    fn window_width(app: &mut App) -> f32 {
        let mut numbers = app.world_mut().query::<(&ui::roll::RollNumber, &Node)>();
        let (_, node) = numbers.single(app.world()).expect("one number");
        let Val::Px(width) = node.width else {
            panic!("the window width is in pixels")
        };
        width
    }

    #[test]
    fn a_static_text_and_a_rolled_number_share_the_same_line_box() {
        let mut app = app_with_layout();
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
        let mut app = app_with_layout();
        let mut widths = vec![window_width(&mut app)];
        show(&mut app, &[10]);
        for _ in 0..10 {
            app.update();
            widths.push(window_width(&mut app));
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
