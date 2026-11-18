use bevy::prelude::*;

use crate::ui;

pub fn drive_roll_machines(
    time: Res<Time>,
    mut machines: Query<(Entity, &mut ui::roll::RollMachine)>,
    mut numbers: Query<(&mut ui::roll::RollNumber, &Children)>,
    mut columns: Query<(&Children, &mut UiTransform), With<ui::roll::RollNumberColumn>>,
    mut rows: Query<(&ui::roll::RollNumberRow, &mut Text)>,
) {
    let delta_secs = time.into_inner().delta_secs();
    for (pill, mut machine) in &mut machines {
        for output in machine.update(ui::roll::Input::Tick { delta_secs }) {
            match output {
                ui::roll::Output::RollFrame { from, to, progress } => {
                    ui::roll::bevy_systems::apply_roll_numbers(
                        pill,
                        &from,
                        &to,
                        progress,
                        &mut numbers,
                        &mut columns,
                        &mut rows,
                    );
                }
                ui::roll::Output::RollSettled { numbers: settled } => {
                    ui::roll::bevy_systems::apply_roll_numbers(
                        pill,
                        &settled,
                        &settled,
                        1.0,
                        &mut numbers,
                        &mut columns,
                        &mut rows,
                    );
                }
                ui::roll::Output::RevealStarted { .. }
                | ui::roll::Output::RevealFrame { .. }
                | ui::roll::Output::RevealSettled { .. } => {}
            }
        }
        match &machine.phase {
            ui::roll::Phase::Idle => {
                let settled = &machine.numbers;
                ui::roll::bevy_systems::apply_roll_numbers(
                    pill,
                    settled,
                    settled,
                    1.0,
                    &mut numbers,
                    &mut columns,
                    &mut rows,
                );
            }
            ui::roll::Phase::Reveal {
                revealed_numbers, ..
            } => {
                ui::roll::bevy_systems::apply_roll_numbers(
                    pill,
                    revealed_numbers,
                    revealed_numbers,
                    1.0,
                    &mut numbers,
                    &mut columns,
                    &mut rows,
                );
            }
            ui::roll::Phase::Roll { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::drive_roll_machines;
    use crate::ui;
    use ui::roll::NumberStyle;

    const ROW_HEIGHT: f32 = 10.0;
    const FRAME_MILLIS: u64 = 50;

    const STYLE: NumberStyle = NumberStyle {
        font_size: 11.0,
        row_height: ROW_HEIGHT,
        text_color: Color::WHITE,
    };

    #[derive(Component)]
    struct StaticText;

    fn app_with_two_numbers() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            FRAME_MILLIS,
        )));
        app.add_systems(Startup, spawn_fixture);
        app.add_systems(Update, drive_roll_machines);
        app.update();
        app
    }

    fn spawn_fixture(mut commands: Commands) {
        let pill = commands
            .spawn(ui::roll::RollMachine(ui::roll::State::new(
                vec![9, 9],
                None,
            )))
            .id();
        commands.entity(pill).with_children(|machine| {
            ui::roll::bevy_systems::spawn_roll_number(&mut machine.reborrow(), pill, 0, 9, STYLE);
            ui::roll::bevy_systems::spawn_roll_number(&mut machine.reborrow(), pill, 1, 9, STYLE);
        });
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
        let found = numbers
            .iter(app.world())
            .find(|(_, number)| number.slot == slot);
        found.map(|(entity, _)| entity).expect("the number window")
    }

    fn number_value(app: &mut App, slot: usize) -> u32 {
        let mut numbers = app.world_mut().query::<&ui::roll::RollNumber>();
        let found = numbers.iter(app.world()).find(|number| number.slot == slot);
        found.map(|number| number.value).expect("the number window")
    }

    fn column_entity(app: &mut App, window: Entity) -> Entity {
        let mut columns = app
            .world_mut()
            .query::<(Entity, &ui::roll::RollNumberColumn, &ChildOf)>();
        let found = columns
            .iter(app.world())
            .find(|(_, _, parent)| parent.parent() == window);
        found
            .map(|(entity, _, _)| entity)
            .expect("the number column")
    }

    fn column_offset(app: &mut App, window: Entity) -> f32 {
        let column = column_entity(app, window);
        let mut columns = app.world_mut().query::<&UiTransform>();
        let transform = columns.get(app.world(), column).expect("the number column");
        let Val::Px(offset) = transform.translation.y else {
            panic!("the column translates in pixels")
        };
        offset
    }

    fn row_texts(app: &mut App, window: Entity) -> [String; 2] {
        let column = column_entity(app, window);
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
    fn test_usage() {
        let mut app = app_with_two_numbers();
        let friends = number_entity(&mut app, 1);
        assert_eq!(number_value(&mut app, 1), 9);
        assert_eq!(column_offset(&mut app, friends), 0.0);
        assert_eq!(
            row_texts(&mut app, friends),
            [String::from("9"), String::from("9")]
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
        assert!(matches!(app_state(&mut app).phase, ui::roll::Phase::Idle));
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

    fn app_state(app: &mut App) -> ui::roll::State {
        let mut machines = app.world_mut().query::<&ui::roll::RollMachine>();
        let found = machines.iter(app.world()).next();
        found
            .map(|machine| ui::roll::State::clone(machine))
            .expect("the machine")
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
        app.add_systems(Startup, spawn_layout_fixture);
        app.add_plugins(ui::roll::UiPlugin);
        app.finish();
        app.cleanup();
        app.update();
        app
    }

    fn spawn_layout_fixture(mut commands: Commands) {
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
            StaticText,
            Text::new("8"),
            STYLE.text_font(),
            STYLE.line_height(),
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
            ui::roll::bevy_systems::spawn_roll_number(&mut parent.reborrow(), pill, 0, 9, STYLE);
        });
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
            .query_filtered::<&ComputedNode, With<StaticText>>();
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
            assert!((height - ROW_HEIGHT).abs() < 0.01, "line box {height}");
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

    const PILL_STYLE: NumberStyle = NumberStyle {
        font_size: 11.0,
        row_height: 13.0,
        text_color: Color::WHITE,
    };

    const FRIEND_STYLE: NumberStyle = NumberStyle {
        font_size: 9.0,
        row_height: 11.0,
        text_color: Color::WHITE,
    };

    const PREVIEW_ROW_COLOR: Color = Color::srgb(0.10, 0.11, 0.14);
    const PREVIEW_BUBBLE_COLOR: Color = Color::srgb(0.16, 0.44, 0.24);
    const PREVIEW_BADGE_COLOR: Color = Color::srgb(0.35, 0.80, 0.44);
    const PREVIEW_DIVIDER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.60);

    #[derive(Resource, Default, Deref)]
    struct PreviewClock(Option<f32>);

    fn start_preview(world: &mut World) {
        let now = world.resource::<Time>().elapsed_secs();
        world.insert_resource(PreviewClock(Some(now)));
    }

    fn script_numbers(
        time: Res<Time>,
        clock: Res<PreviewClock>,
        mut machines: Query<&mut ui::roll::RollMachine>,
    ) {
        let Some(started_at) = **clock.into_inner() else {
            return;
        };
        let elapsed = time.into_inner().elapsed_secs() - started_at;
        let numbers = if elapsed < 1.55 {
            if elapsed < 0.15 {
                vec![2, 8, 9]
            } else {
                vec![3, 8, 9]
            }
        } else if elapsed < 2.95 {
            vec![3, 8, 10]
        } else {
            vec![3, 8, 11]
        };
        for mut machine in &mut machines {
            if machine.target == numbers {
                continue;
            }
            let _ = machine.update(ui::roll::Input::Show {
                numbers: numbers.clone(),
            });
        }
    }

    fn spawn_preview_label(parent: &mut ChildSpawnerCommands, label: &str) {
        parent.spawn((
            Text::new(label),
            TextFont::from_font_size(FontSize::Px(8.0)),
            TextColor(Color::srgb(0.70, 0.70, 0.70)),
        ));
    }

    fn spawn_preview_static(parent: &mut ChildSpawnerCommands, value: &str, style: NumberStyle) {
        parent.spawn((
            Text::new(value),
            style.text_font(),
            style.line_height(),
            TextColor(style.text_color),
            TextLayout::no_wrap(),
        ));
    }

    fn spawn_preview_divider(parent: &mut ChildSpawnerCommands) {
        parent.spawn((
            Node {
                width: Val::Px(1.0),
                height: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(PREVIEW_DIVIDER_COLOR),
        ));
    }

    fn spawn_preview_friend_icon(parent: &mut ChildSpawnerCommands) {
        parent
            .spawn(Node {
                width: Val::Px(10.0),
                height: Val::Px(9.0),
                ..default()
            })
            .with_children(|icon| {
                icon.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(3.0),
                        width: Val::Px(4.0),
                        height: Val::Px(4.0),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(PREVIEW_BADGE_COLOR),
                ));
                icon.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(5.0),
                        width: Val::Px(10.0),
                        height: Val::Px(4.0),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(PREVIEW_BADGE_COLOR),
                ));
            });
    }

    fn spawn_preview_row(parent: &mut ChildSpawnerCommands, is_widget: bool) {
        let mut row = parent.spawn((
            Node {
                width: Val::Px(232.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                column_gap: Val::Px(6.0),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(PREVIEW_ROW_COLOR),
        ));
        if is_widget {
            row.insert(ui::roll::RollMachine(ui::roll::State::new(
                vec![2, 8, 9],
                None,
            )));
        }
        let row_id = row.id();
        row.with_children(|row| {
            row.spawn(Node {
                width: Val::Px(128.0),
                overflow: Overflow::clip_x(),
                ..default()
            })
            .with_children(|clip| {
                clip.spawn((
                    Text::new("alpha"),
                    TextFont::from_font_size(FontSize::Px(11.0)),
                    TextColor(Color::WHITE),
                ));
            });
            row.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::axes(Val::Px(7.0), Val::Px(2.5)),
                    column_gap: Val::Px(4.0),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(PREVIEW_BUBBLE_COLOR),
            ))
            .with_children(|bubble| {
                spawn_preview_friend_icon(bubble);
                if is_widget {
                    ui::roll::bevy_systems::spawn_roll_number(bubble, row_id, 2, 9, FRIEND_STYLE);
                } else {
                    spawn_preview_static(bubble, "11", FRIEND_STYLE);
                }
                spawn_preview_divider(bubble);
                bubble
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|occupancy| {
                        if is_widget {
                            ui::roll::bevy_systems::spawn_roll_number(
                                occupancy, row_id, 0, 2, PILL_STYLE,
                            );
                        } else {
                            spawn_preview_static(occupancy, "3", PILL_STYLE);
                        }
                        spawn_preview_static(occupancy, "/", PILL_STYLE);
                        if is_widget {
                            ui::roll::bevy_systems::spawn_roll_number(
                                occupancy, row_id, 1, 8, PILL_STYLE,
                            );
                        } else {
                            spawn_preview_static(occupancy, "8", PILL_STYLE);
                        }
                    });
            });
        });
    }

    fn spawn_preview_rows(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.entity(root).with_children(|root| {
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|column| {
                spawn_preview_row(column, true);
                spawn_preview_label(column, "widget");
                spawn_preview_row(column, false);
                spawn_preview_label(column, "today (plain text)");
            });
        });
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_build(app: &mut App) {
        app.init_resource::<PreviewClock>();
        app.add_systems(Startup, spawn_preview_rows);
        app.add_systems(
            Update,
            (
                script_numbers.before(ui::roll::RollSet::Drive),
                drive_roll_machines.in_set(ui::roll::RollSet::Drive),
            ),
        );
        app.add_systems(
            PostUpdate,
            ui::roll::bevy_systems::resize_roll_numbers.after(bevy::ui::UiSystems::Layout),
        );
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
    }

    #[test]
    fn drive_roll_machines_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("roll_pill"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("friends 9 to 11, occupants 2 to 3"),
                apply: preview_noop,
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("whole-number slide, widget over plain text"),
                frames: 138,
                fps: 30,
                apply: start_preview,
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
