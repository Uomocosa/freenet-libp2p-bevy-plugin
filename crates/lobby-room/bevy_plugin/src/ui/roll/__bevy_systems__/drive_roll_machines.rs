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
    use bevy::prelude::*;

    use super::drive_roll_machines;
    use crate::ui;
    use ui::roll::NumberStyle;

    const PILL_STYLE: NumberStyle = NumberStyle {
        font_size: 11.0,
        row_height: 13.0,
        text_color: Color::WHITE,
    };

    const PREVIEW_BUBBLE_COLOR: Color = Color::srgb(0.16, 0.44, 0.24);

    #[test]
    fn drive_roll_machines_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("roll_pill"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    let pill = commands
                        .spawn((
                            ChildOf(root),
                            ui::roll::RollMachine(ui::roll::State::new(vec![2, 8], None)),
                            Node {
                                flex_direction: FlexDirection::Row,
                                padding: UiRect::axes(Val::Px(7.0), Val::Px(2.5)),
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(PREVIEW_BUBBLE_COLOR),
                        ))
                        .id();
                    commands.entity(pill).with_children(|bubble| {
                        ui::roll::bevy_systems::spawn_roll_number(bubble, pill, 0, 2, PILL_STYLE);
                        bubble.spawn((
                            Text::new("/"),
                            PILL_STYLE.text_font(),
                            PILL_STYLE.line_height(),
                            TextColor(PILL_STYLE.text_color),
                            TextLayout::no_wrap(),
                        ));
                        ui::roll::bevy_systems::spawn_roll_number(bubble, pill, 1, 8, PILL_STYLE);
                    });
                });
                app.add_systems(
                    Update,
                    (|time: Res<Time>, mut machines: Query<&mut ui::roll::RollMachine>| {
                        let elapsed = time.elapsed_secs() % 3.6;
                        let numbers = if elapsed < 1.2 {
                            vec![2, 8]
                        } else if elapsed < 2.4 {
                            vec![3, 8]
                        } else {
                            vec![3, 9]
                        };
                        for mut machine in &mut machines {
                            if machine.target == numbers {
                                continue;
                            }
                            let _ = machine.update(ui::roll::Input::Show {
                                numbers: numbers.clone(),
                            });
                        }
                    })
                    .before(ui::roll::RollSet::Drive),
                );
                app.add_systems(Update, drive_roll_machines.in_set(ui::roll::RollSet::Drive));
                app.add_systems(
                    PostUpdate,
                    ui::roll::bevy_systems::resize_roll_numbers.after(bevy::ui::UiSystems::Layout),
                );
                app.add_systems(
                    PostStartup,
                    lele_bevy_preview::scene::bevy_systems::center_roots,
                );
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("occupants 2 of 8, friends 3 to 9"),
                apply: |_world: &mut World| {},
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("whole-number slide"),
                frames: 108,
                fps: 30,
                apply: |_world: &mut World| {},
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
