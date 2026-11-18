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
    use derive_more::Deref;

    #[derive(Resource, Deref)]
    struct Wanted(core_discovery::LobbyRow);

    fn update(
        row: Res<Wanted>,
        mut rows: Query<(
            &ui::discovery::RoomButton,
            &mut ui::roll::RollMachine,
            &ui::discovery::RoomRowParts,
        )>,
        mut texts: Query<&mut Text>,
        mut backgrounds: Query<&mut BackgroundColor>,
    ) {
        update_room_row(row.into_inner(), &mut rows, &mut texts, &mut backgrounds);
    }

    fn row_fixture(
        occupants: usize,
        occupancy: core_discovery::RoomOccupancy,
    ) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants,
            room_size: core_discovery::RoomSize::try_from(8).expect("room size"),
            occupancy,
            friends: 0,
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(Wanted(row_fixture(4, core_discovery::RoomOccupancy::Full)));
        app.add_systems(Startup, |mut commands: Commands| {
            let name_text = commands.spawn(Text::new("old")).id();
            let bubble = commands
                .spawn((
                    ui::discovery::RoomBubble,
                    BackgroundColor(ui::discovery::ROOM_BUBBLE_EMPTY_COLOR),
                ))
                .id();
            commands.spawn((
                ui::discovery::RoomButton(core_discovery::RoomName::from("alpha")),
                ui::roll::RollMachine(ui::roll::State::new(vec![3, 8, 0], Some(2))),
                ui::discovery::RoomRowParts { name_text, bubble },
            ));
        });
        app.add_systems(Update, update);
        app.update();

        let mut machines = app.world_mut().query::<&ui::roll::RollMachine>();
        let machine = machines.single(app.world()).expect("one machine");
        assert_eq!(machine.target, vec![4, 8, 0]);
        assert_eq!(machine.numbers, vec![3, 8, 0]);

        let mut texts = app.world_mut().query::<&Text>();
        assert_eq!(
            texts.single(app.world()).expect("name text").as_str(),
            "alpha"
        );

        let mut backgrounds = app
            .world_mut()
            .query_filtered::<&BackgroundColor, With<ui::discovery::RoomBubble>>();
        let BackgroundColor(color) = backgrounds.single(app.world()).expect("bubble colour");
        assert_eq!(*color, ui::discovery::ROOM_BUBBLE_FULL_COLOR);
    }

    fn spawn_preview_row(mut commands: Commands) {
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
    }

    fn preview_start(world: &mut World) {
        world.insert_resource(Wanted(row_fixture(
            3,
            core_discovery::RoomOccupancy::HasSpace,
        )));
    }

    fn preview_full(world: &mut World) {
        world.insert_resource(Wanted(row_fixture(4, core_discovery::RoomOccupancy::Full)));
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(Wanted(row_fixture(
            3,
            core_discovery::RoomOccupancy::HasSpace,
        )));
        app.add_plugins(ui::roll::UiPlugin);
        app.add_systems(Startup, spawn_preview_row);
        app.add_systems(Update, update);
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
    }

    #[test]
    fn update_room_row_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("update_room_row"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("three of eight"),
                    apply: preview_start,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("four of four"),
                    apply: preview_full,
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
