use crate::ui;
use bevy::prelude::*;

use ui::basic;
use ui::discovery::{
    HOVER_OUTLINE_WIDTH, RoomButton, SELECTED_ROOM_PREVIEW_OUTLINE_COLOR, STREAMED_ROOM_BG_COLOR,
    STREAMED_ROOM_OUTLINE_COLOR, STREAMED_ROOM_OUTLINE_WIDTH, SelectedRoom, StreamedRoom,
};

pub fn highlight_selected_room(
    selection: Res<SelectedRoom>,
    streamed: Res<StreamedRoom>,
    mut rows: Query<(&RoomButton, &mut Outline, &mut BackgroundColor)>,
) {
    let selection = selection.into_inner();
    let streamed = streamed.into_inner();
    let selected_room = (**selection).as_ref();
    let streamed_room = (**streamed).as_ref();
    for (button, mut outline, mut background) in &mut rows {
        let is_selected = selected_room == Some(&**button);
        let is_streamed = streamed_room == Some(&**button);
        outline.width = Val::Px(if is_streamed {
            STREAMED_ROOM_OUTLINE_WIDTH
        } else {
            HOVER_OUTLINE_WIDTH
        });
        if is_streamed {
            outline.color = STREAMED_ROOM_OUTLINE_COLOR;
        } else if is_selected {
            outline.color = SELECTED_ROOM_PREVIEW_OUTLINE_COLOR;
        }
        let BackgroundColor(existing) = &mut *background;
        let background_color = if is_streamed {
            STREAMED_ROOM_BG_COLOR
        } else {
            basic::constants::ROOM_COLOR
        };
        if *existing != background_color {
            *existing = background_color;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::highlight_selected_room;

    #[test]
    fn highlight_selected_room_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("highlight_selected_room"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(ui::discovery::SelectedRoom::default());
                app.insert_resource(ui::discovery::StreamedRoom::default());
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    commands
                        .spawn((
                            ChildOf(root),
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                ..default()
                            },
                        ))
                        .with_children(|list| {
                            for name in ["alpha", "beta"] {
                                let row = core_discovery::LobbyRow {
                                    name: core_discovery::RoomName::from(name),
                                    occupants: 2,
                                    room_size: core_discovery::RoomSize::try_from(8).expect("8"),
                                    occupancy: core_discovery::RoomOccupancy::HasSpace,
                                    friends: 0,
                                };
                                ui::discovery::bevy_systems::spawn_room_row(list, &row);
                            }
                        });
                });
                app.add_systems(
                    Update,
                    (
                        ui::discovery::bevy_systems::animate_hover,
                        highlight_selected_room,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("hover preview"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<ui::discovery::SelectedRoom>() =
                            Some(core_discovery::RoomName::from("alpha"));
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("streamed room"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<ui::discovery::StreamedRoom>() =
                            Some(core_discovery::RoomName::from("alpha"));
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
