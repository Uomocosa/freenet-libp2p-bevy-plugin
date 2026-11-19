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
    use crate::discovery as core_discovery;
    use crate::ui;

    const ROW_COLOR: Color = Color::srgb(0.10, 0.11, 0.14);
    const ROW_WIDTH: f32 = 232.0;
    const NAME_PX: f32 = 11.0;
    const LABEL_PX: f32 = 8.0;
    const SCRIPT: [(f32, usize, usize); 4] = [
        (0.15, 2, 9),
        (1.55, 3, 9),
        (2.95, 3, 10),
        (f32::INFINITY, 3, 11),
    ];
    const TIMELINE_LABEL: &str =
        "occupants 2→3, friends 9→10→11 (pill grows); widget over plain text";

    #[test]
    fn drive_roll_machines_ui_mp4_preview() {
        lele_bevy_preview::scene::Scene {
            name: String::from("roll_pill"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
                    name: core_discovery::RoomName::from("alpha"),
                    occupants: 2,
                    room_size: core_discovery::RoomSize::try_from(8).expect("room size"),
                    occupancy: core_discovery::RoomOccupancy::HasSpace,
                    friends: 9,
                }]));
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    commands.entity(root).insert(children![
                        (
                            ui::discovery::RoomList,
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                ..default()
                            },
                        ),
                        (
                            Text::new("widget"),
                            TextFont::from_font_size(FontSize::Px(LABEL_PX)),
                        ),
                        (
                            Node {
                                width: Val::Px(ROW_WIDTH),
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(ROW_COLOR),
                            children![(
                                Text::new("alpha          11   3/8"),
                                TextFont::from_font_size(FontSize::Px(NAME_PX)),
                            )],
                        ),
                        (
                            Text::new("plain text"),
                            TextFont::from_font_size(FontSize::Px(LABEL_PX)),
                        ),
                    ]);
                });
                app.configure_sets(
                    Update,
                    ui::roll::RollSet::Drive.after(ui::discovery::bevy_systems::rebuild_room_list),
                );
                app.add_systems(
                    Update,
                    (
                        (|clock: Res<lele_bevy_preview::scene::Clock>,
                          time: Res<Time>,
                          mut rows: ResMut<core_discovery::LobbyRows>| {
                            let Some(elapsed) = clock.elapsed(&time) else {
                                return;
                            };
                            let (occupants, friends) = SCRIPT
                                .into_iter()
                                .find(|(limit, _, _)| elapsed < *limit)
                                .map_or((3, 11), |(_, occupants, friends)| (occupants, friends));
                            let core_discovery::LobbyRows(list) = &mut *rows;
                            if let Some(row) = list.first_mut()
                                && (row.occupants, row.friends) != (occupants, friends)
                            {
                                (row.occupants, row.friends) = (occupants, friends);
                            }
                        })
                        .before(ui::discovery::bevy_systems::rebuild_room_list),
                        ui::discovery::bevy_systems::rebuild_room_list,
                    ),
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
                label: String::from("alpha: 2 of 8, 9 friends"),
                apply: |_world: &mut World| {},
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from(TIMELINE_LABEL),
                frames: 108,
                fps: 30,
                apply: lele_bevy_preview::scene::Clock::start,
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview"),
            ..Default::default()
        })
        .expect("preview");
    }
}
