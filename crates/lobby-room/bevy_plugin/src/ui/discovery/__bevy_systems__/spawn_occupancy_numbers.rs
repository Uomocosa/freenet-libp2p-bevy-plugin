use crate::ui;
use bevy::prelude::*;

use ui::discovery::{OCCUPANTS_SLOT, ROOM_NUMBER_STYLE, ROOM_SIZE_SLOT};

pub fn spawn_occupancy_numbers(
    pill: &mut ChildSpawnerCommands,
    row: Entity,
    occupants: u32,
    room_size: u32,
) {
    pill.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|occupancy| {
        ui::roll::bevy_systems::spawn_roll_number(
            occupancy,
            row,
            OCCUPANTS_SLOT,
            occupants,
            ROOM_NUMBER_STYLE,
        );
        occupancy.spawn((
            Text::new("/"),
            ROOM_NUMBER_STYLE.text_font(),
            ROOM_NUMBER_STYLE.line_height(),
            TextColor(ROOM_NUMBER_STYLE.text_color),
        ));
        ui::roll::bevy_systems::spawn_roll_number(
            occupancy,
            row,
            ROOM_SIZE_SLOT,
            room_size,
            ROOM_NUMBER_STYLE,
        );
    });
}

#[cfg(test)]
mod tests {
    use super::spawn_occupancy_numbers;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn spawn_occupancy_numbers_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_occupancy_numbers"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_plugins(ui::roll::UiPlugin);
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    let row = commands.spawn_empty().id();
                    commands
                        .spawn((
                            ChildOf(root),
                            ui::discovery::RoomBubble,
                            Node {
                                flex_direction: FlexDirection::Row,
                                padding: UiRect::axes(Val::Px(7.0), Val::Px(2.5)),
                                column_gap: Val::Px(4.0),
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(ui::discovery::ROOM_BUBBLE_HAS_SPACE_COLOR),
                        ))
                        .with_children(|pill| {
                            spawn_occupancy_numbers(pill, row, 2, 8);
                        });
                });
                app.add_systems(
                    PostStartup,
                    lele_bevy_preview::scene::bevy_systems::center_roots,
                );
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("two of eight"),
                apply: |_world: &mut World| {},
            }],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
