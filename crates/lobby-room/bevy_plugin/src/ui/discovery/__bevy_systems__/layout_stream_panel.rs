use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    RoomPreviewPanel, RoomStreamPanel, STREAM_BOTTOM, STREAM_FRAME_PADDING, STREAM_LEFT,
    STREAM_RADIUS, STREAM_RIGHT, STREAM_TOP, StreamDisplay,
};

type StreamPanelNode<'w, 's> =
    Single<'w, 's, &'static mut Node, (With<RoomStreamPanel>, Without<RoomPreviewPanel>)>;
type PreviewVisibility<'w, 's> =
    Single<'w, 's, &'static mut Visibility, (With<RoomPreviewPanel>, Without<RoomStreamPanel>)>;

pub fn layout_stream_panel(
    display: Res<StreamDisplay>,
    panel: Option<StreamPanelNode>,
    preview: Option<PreviewVisibility>,
) {
    let display = display.into_inner();
    let is_fullscreen = *display == StreamDisplay::Fullscreen;
    if let Some(panel) = panel {
        let mut node = panel.into_inner();
        let (top, left, right, bottom, radius, padding) = if is_fullscreen {
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        } else {
            (
                STREAM_TOP,
                STREAM_LEFT,
                STREAM_RIGHT,
                STREAM_BOTTOM,
                STREAM_RADIUS,
                STREAM_FRAME_PADDING,
            )
        };
        if node.top != Val::Px(top) {
            node.top = Val::Px(top);
        }
        if node.left != Val::Px(left) {
            node.left = Val::Px(left);
        }
        if node.right != Val::Px(right) {
            node.right = Val::Px(right);
        }
        if node.bottom != Val::Px(bottom) {
            node.bottom = Val::Px(bottom);
        }
        let wanted_radius = BorderRadius::all(Val::Px(radius));
        if node.border_radius != wanted_radius {
            node.border_radius = wanted_radius;
        }
        let wanted_padding = UiRect::all(Val::Px(padding));
        if node.padding != wanted_padding {
            node.padding = wanted_padding;
        }
    }
    let Some(preview) = preview else {
        return;
    };
    if is_fullscreen {
        *preview.into_inner() = Visibility::Hidden;
    }
}

#[cfg(test)]
mod png_tests {
    use super::layout_stream_panel;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    const ROOM_NAMES: [&str; 3] = ["alpha", "beta", "gamma"];

    #[test]
    fn layout_stream_panel_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("layout_stream_panel"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: |app: &mut App| {
                let rows: Vec<core_discovery::LobbyRow> = ROOM_NAMES
                    .iter()
                    .map(|name| core_discovery::LobbyRow {
                        name: core_discovery::RoomName::from(*name),
                        occupants: 2,
                        room_size: core_discovery::RoomSize::try_from(4).expect("4"),
                        occupancy: core_discovery::RoomOccupancy::HasSpace,
                        friends: 0,
                    })
                    .collect();
                app.insert_resource(ui::discovery::StreamDisplay::default());
                app.insert_resource(ui::discovery::StreamedRoom(Some(
                    core_discovery::RoomName::from("alpha"),
                )));
                app.insert_resource(ui::discovery::SelectedRoom(Some(
                    core_discovery::RoomName::from("beta"),
                )));
                app.insert_resource(core_discovery::LobbyRows(rows.clone()));
                app.add_systems(
                    Startup,
                    (
                        ui::discovery::bevy_systems::spawn_room_stream,
                        ui::discovery::bevy_systems::spawn_room_preview,
                        move |mut commands: Commands| {
                            commands
                                .spawn((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        top: Val::Px(48.0),
                                        right: Val::Px(ui::discovery::LOBBY_PANEL_RIGHT),
                                        width: Val::Px(ui::discovery::LOBBY_PANEL_WIDTH),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(4.0),
                                        padding: UiRect::all(Val::Px(8.0)),
                                        ..default()
                                    },
                                    BackgroundColor(ui::discovery::PANEL_COLOR),
                                ))
                                .with_children(|list| {
                                    for row in &rows {
                                        ui::discovery::bevy_systems::spawn_room_row(list, row);
                                    }
                                });
                        },
                    ),
                );
                app.add_systems(
                    Update,
                    (
                        ui::discovery::bevy_systems::refresh_room_preview,
                        ui::discovery::bevy_systems::refresh_room_stream,
                        layout_stream_panel,
                        ui::discovery::bevy_systems::place_room_preview,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("panel with preview"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("fullscreen"),
                    apply: |world: &mut World| {
                        *world.resource_mut::<ui::discovery::StreamDisplay>() =
                            ui::discovery::StreamDisplay::Fullscreen;
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
