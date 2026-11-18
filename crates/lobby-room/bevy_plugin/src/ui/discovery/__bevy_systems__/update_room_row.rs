use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

pub fn update_room_row(
    row: &core_discovery::LobbyRow,
    rows: &mut Query<(
        &ui::discovery::RoomButton,
        &mut ui::roll::RollMachine,
        &ui::discovery::RoomRowParts,
    )>,
    texts: &mut Query<&mut Text>,
    backgrounds: &mut Query<&mut BackgroundColor>,
) {
    let [occupants, room_size, friends] = ui::discovery::bevy_systems::room_numbers(row);
    let Some((_, mut machine, parts)) = rows
        .iter_mut()
        .find(|(button, _, _)| button.as_str() == row.name.as_str())
    else {
        return;
    };
    let _ = machine.update(ui::roll::Input::Show {
        numbers: vec![occupants, room_size, friends],
    });
    if let Ok(mut text) = texts.get_mut(parts.name_text) {
        *text = Text::new(row.name.as_str());
    }
    if let Ok(mut background) = backgrounds.get_mut(parts.bubble) {
        *background = BackgroundColor(ui::discovery::bevy_systems::room_bubble_color(
            row.occupancy,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::update_room_row;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn update_room_row_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("update_room_row"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
                    name: core_discovery::RoomName::from("alpha"),
                    occupants: 3,
                    room_size: core_discovery::RoomSize::try_from(8).expect("room size"),
                    occupancy: core_discovery::RoomOccupancy::HasSpace,
                    friends: 0,
                }]));
                app.add_plugins(ui::roll::UiPlugin);
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    let name_text = commands
                        .spawn((ChildOf(root), Text::new("alpha"), TextColor(Color::WHITE)))
                        .id();
                    let bubble = commands
                        .spawn((
                            ChildOf(root),
                            ui::discovery::RoomBubble,
                            Node {
                                padding: UiRect::axes(Val::Px(7.0), Val::Px(2.5)),
                                ..default()
                            },
                            BackgroundColor(ui::discovery::ROOM_BUBBLE_HAS_SPACE_COLOR),
                        ))
                        .id();
                    commands.spawn((
                        ChildOf(root),
                        ui::discovery::RoomButton(core_discovery::RoomName::from("alpha")),
                        ui::roll::RollMachine(ui::roll::State::new(vec![3, 8, 0], Some(2))),
                        ui::discovery::RoomRowParts { name_text, bubble },
                    ));
                });
                app.add_systems(
                    Update,
                    |wanted: Res<core_discovery::LobbyRows>,
                     mut rows: Query<(
                        &ui::discovery::RoomButton,
                        &mut ui::roll::RollMachine,
                        &ui::discovery::RoomRowParts,
                    )>,
                     mut texts: Query<&mut Text>,
                     mut backgrounds: Query<&mut BackgroundColor>| {
                        if let Some(row) = wanted.first() {
                            update_room_row(row, &mut rows, &mut texts, &mut backgrounds);
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
                    label: String::from("three of eight"),
                    apply: |world: &mut World| {
                        *world.resource_mut::<core_discovery::LobbyRows>() =
                            core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
                                name: core_discovery::RoomName::from("alpha"),
                                occupants: 3,
                                room_size: core_discovery::RoomSize::try_from(8)
                                    .expect("room size"),
                                occupancy: core_discovery::RoomOccupancy::HasSpace,
                                friends: 0,
                            }]);
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("four of four"),
                    apply: |world: &mut World| {
                        *world.resource_mut::<core_discovery::LobbyRows>() =
                            core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
                                name: core_discovery::RoomName::from("alpha"),
                                occupants: 4,
                                room_size: core_discovery::RoomSize::try_from(8)
                                    .expect("room size"),
                                occupancy: core_discovery::RoomOccupancy::Full,
                                friends: 0,
                            }]);
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
