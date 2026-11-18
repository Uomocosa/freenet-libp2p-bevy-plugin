use bevy::prelude::*;

use crate::ui;

pub fn apply_roll_numbers(
    pill: Entity,
    from: &[u32],
    to: &[u32],
    progress: f32,
    numbers: &mut Query<(&mut ui::roll::RollNumber, &Children)>,
    columns: &mut Query<(&Children, &mut UiTransform), With<ui::roll::RollNumberColumn>>,
    rows: &mut Query<(&ui::roll::RollNumberRow, &mut Text)>,
) {
    let eased = ui::roll::bevy_systems::ease_out_cubic(progress);
    let is_settled = progress >= 1.0;
    for (mut number, window_children) in numbers.iter_mut() {
        if number.pill != pill {
            continue;
        }
        let (Some(&start), Some(&end)) = (from.get(number.slot), to.get(number.slot)) else {
            continue;
        };
        let Some(column) = window_children.iter().next() else {
            continue;
        };
        let Ok((column_children, mut transform)) = columns.get_mut(column) else {
            continue;
        };
        for child in column_children.iter() {
            let Ok((row, mut text)) = rows.get_mut(child) else {
                continue;
            };
            let wanted = match *row {
                ui::roll::RollNumberRow::Outgoing if is_settled => end,
                ui::roll::RollNumberRow::Outgoing => start,
                ui::roll::RollNumberRow::Incoming => end,
            }
            .to_string();
            if text.as_str() != wanted {
                *text = Text::new(wanted);
            }
        }
        let offset = if is_settled {
            0.0
        } else {
            -(eased * number.row_height)
        };
        transform.translation = Val2::px(0.0, offset);
        if is_settled {
            number.value = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::apply_roll_numbers;
    use crate::ui;
    use ui::roll::NumberStyle;

    const STYLE: NumberStyle = NumberStyle {
        font_size: 11.0,
        row_height: 10.0,
        text_color: Color::WHITE,
    };

    #[test]
    fn apply_roll_numbers_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("apply_roll_numbers"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    let pill = commands
                        .spawn((
                            ChildOf(root),
                            ui::roll::RollMachine(ui::roll::State::new(vec![9], None)),
                            Node {
                                flex_direction: FlexDirection::Row,
                                padding: UiRect::axes(Val::Px(7.0), Val::Px(2.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.16, 0.44, 0.24)),
                        ))
                        .id();
                    commands.entity(pill).with_children(|parent| {
                        ui::roll::bevy_systems::spawn_roll_number(parent, pill, 0, 9, STYLE);
                    });
                });
                app.add_systems(
                    Update,
                    |machine: Single<&ui::roll::RollMachine>,
                     mut numbers: Query<(&mut ui::roll::RollNumber, &Children)>,
                     mut columns: Query<
                        (&Children, &mut UiTransform),
                        With<ui::roll::RollNumberColumn>,
                    >,
                     mut rows: Query<(&ui::roll::RollNumberRow, &mut Text)>| {
                        let progress = match &machine.phase {
                            ui::roll::Phase::Roll { elapsed_secs, .. } => {
                                (elapsed_secs / ui::roll::ROLL_SECS).clamp(0.0, 1.0)
                            }
                            _ => 1.0,
                        };
                        let pills: Vec<Entity> =
                            numbers.iter_mut().map(|(number, _)| number.pill).collect();
                        for pill in pills {
                            apply_roll_numbers(
                                pill,
                                &[9],
                                &[10],
                                progress,
                                &mut numbers,
                                &mut columns,
                                &mut rows,
                            );
                        }
                    },
                );
                app.add_systems(
                    PostStartup,
                    lele_bevy_preview::scene::bevy_systems::center_roots,
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("start"),
                    apply: |world: &mut World| {
                        let mut machines = world.query::<&mut ui::roll::RollMachine>();
                        for mut machine in machines.iter_mut(world) {
                            machine.phase = ui::roll::Phase::Roll {
                                from: vec![9],
                                to: vec![10],
                                elapsed_secs: 0.0,
                            };
                        }
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("halfway"),
                    apply: |world: &mut World| {
                        let mut machines = world.query::<&mut ui::roll::RollMachine>();
                        for mut machine in machines.iter_mut(world) {
                            machine.phase = ui::roll::Phase::Roll {
                                from: vec![9],
                                to: vec![10],
                                elapsed_secs: ui::roll::ROLL_SECS / 2.0,
                            };
                        }
                    },
                },
            ],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
