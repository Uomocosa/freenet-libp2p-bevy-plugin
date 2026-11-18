use bevy::prelude::*;

use crate::ui;

pub fn drive_roll_machines(
    time: Res<Time>,
    mut machines: Query<(Entity, &mut ui::roll::RollMachine)>,
    mut numbers: Query<(&mut ui::roll::RollNumber, &Children)>,
    mut digits: Query<(&ui::roll::RollDigit, &mut UiTransform)>,
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
                        &mut digits,
                    );
                }
                ui::roll::Output::RollSettled { numbers: settled } => {
                    ui::roll::bevy_systems::apply_roll_numbers(
                        pill,
                        &settled,
                        &settled,
                        1.0,
                        &mut numbers,
                        &mut digits,
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
                    &mut digits,
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
                    &mut digits,
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
    use ui::roll::RollNumberStyle;

    const ROW_HEIGHT: f32 = 10.0;
    const FRAME_MILLIS: u64 = 50;

    const ONE_DIGIT: RollNumberStyle = RollNumberStyle {
        font_size: 11.0,
        digit_width: 6.0,
        row_height: ROW_HEIGHT,
        digit_count: 1,
        text_color: Color::WHITE,
    };

    const TWO_DIGITS: RollNumberStyle = RollNumberStyle {
        digit_count: 2,
        ..ONE_DIGIT
    };

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
            ui::roll::bevy_systems::spawn_roll_number(
                &mut machine.reborrow(),
                pill,
                0,
                9,
                ONE_DIGIT,
            );
            ui::roll::bevy_systems::spawn_roll_number(
                &mut machine.reborrow(),
                pill,
                1,
                9,
                TWO_DIGITS,
            );
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

    fn digit_offset(app: &mut App, window: Entity, place: usize) -> f32 {
        let mut digits = app
            .world_mut()
            .query::<(&ui::roll::RollDigit, &ChildOf, &UiTransform)>();
        let found = digits
            .iter(app.world())
            .find(|(digit, parent, _)| parent.parent() == window && digit.place == place);
        let (_, _, transform) = found.expect("the digit column");
        let Val::Px(offset) = transform.translation.y else {
            panic!("the digit column translates in pixels")
        };
        offset
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
        assert_offset(digit_offset(&mut app, friends, 0), -90.0);
        show(&mut app, &[9, 10]);
        app.update();
        app.update();
        assert_eq!(number_value(&mut app, 1), 9, "mid-roll the value is stale");
    }

    #[test]
    fn a_roll_moves_both_columns_of_a_two_digit_number() {
        let mut app = app_with_two_numbers();
        let friends = number_entity(&mut app, 1);
        show(&mut app, &[9, 10]);

        app.update();
        assert_offset(digit_offset(&mut app, friends, 0), -91.666_67);
        assert_offset(digit_offset(&mut app, friends, 1), -1.666_666_7);

        app.update();
        app.update();
        assert_offset(digit_offset(&mut app, friends, 0), -95.0);
        assert_offset(digit_offset(&mut app, friends, 1), -5.0);

        for _ in 0..3 {
            app.update();
        }
        assert_eq!(digit_offset(&mut app, friends, 0), 0.0);
        assert_eq!(digit_offset(&mut app, friends, 1), -ROW_HEIGHT);
        assert_eq!(number_value(&mut app, 1), 10);
        assert!(matches!(app_state(&mut app).phase, ui::roll::Phase::Idle));
    }

    #[test]
    fn a_single_digit_number_rolls_its_own_column() {
        let mut app = app_with_two_numbers();
        let occupants = number_entity(&mut app, 0);
        show(&mut app, &[0, 9]);

        for _ in 0..3 {
            app.update();
        }
        assert_offset(digit_offset(&mut app, occupants, 0), -95.0);

        for _ in 0..3 {
            app.update();
        }
        assert_eq!(digit_offset(&mut app, occupants, 0), 0.0);
        assert_eq!(number_value(&mut app, 0), 0);
    }

    fn app_state(app: &mut App) -> ui::roll::State {
        let mut machines = app.world_mut().query::<&ui::roll::RollMachine>();
        let found = machines.iter(app.world()).next();
        found
            .map(|machine| ui::roll::State::clone(machine))
            .expect("the machine")
    }

    fn preview_widgets(mut commands: Commands) {
        let one_digit = RollNumberStyle {
            font_size: 30.0,
            digit_width: 20.0,
            row_height: 34.0,
            ..ONE_DIGIT
        };
        let two_digits = RollNumberStyle {
            digit_count: 2,
            ..one_digit
        };
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        let pill = commands
            .spawn((
                ChildOf(root),
                ui::roll::RollMachine(ui::roll::State::new(vec![9, 9], None)),
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(3.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.16, 0.2, 0.26)),
            ))
            .id();
        commands.entity(pill).with_children(|machine| {
            ui::roll::bevy_systems::spawn_roll_number(
                &mut machine.reborrow(),
                pill,
                0,
                9,
                one_digit,
            );
            ui::roll::bevy_systems::spawn_roll_number(
                &mut machine.reborrow(),
                pill,
                1,
                9,
                two_digits,
            );
        });
    }

    const fn preview_noop(_world: &mut World) {}

    fn show_ten(world: &mut World) {
        let mut machines = world.query::<&mut ui::roll::RollMachine>();
        for mut machine in machines.iter_mut(world) {
            let _ = machine.update(ui::roll::Input::Show {
                numbers: vec![10, 10],
            });
        }
    }

    fn roll_on_to_eleven(time: Res<Time>, mut machines: Query<&mut ui::roll::RollMachine>) {
        if time.into_inner().elapsed_secs() < 0.9 {
            return;
        }
        for mut machine in &mut machines {
            if machine.target.as_slice() == [11, 11] {
                continue;
            }
            let _ = machine.update(ui::roll::Input::Show {
                numbers: vec![11, 11],
            });
        }
    }

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, preview_widgets);
        app.add_systems(Update, (roll_on_to_eleven, drive_roll_machines).chain());
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
    }

    #[test]
    fn drive_roll_machines_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("drive_roll_machines"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("nine and nine"),
                apply: preview_noop,
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("nine to ten to eleven"),
                frames: 60,
                fps: 30,
                apply: show_ten,
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
