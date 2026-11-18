use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    ACCENT_COLOR, INPUT_COLOR, PANEL_COLOR, RoomStreamImage, RoomStreamPanel,
    RoomStreamPlaceholder, RoomStreamViewport, SPINNER_TRACK_COLOR, STREAM_BOTTOM,
    STREAM_FRAME_PADDING, STREAM_IMAGE_PADDING, STREAM_IMAGE_TOP_INSET, STREAM_LEFT,
    STREAM_PLACEHOLDER_SPINNER_SIZE, STREAM_PLACEHOLDER_SPINNER_WIDTH, STREAM_RADIUS, STREAM_RIGHT,
    STREAM_TOP, STREAM_VIEWPORT_RADIUS, STREAM_Z, Screen, SpinnerRing,
};

pub fn spawn_room_stream(mut commands: Commands) {
    commands
        .spawn((
            RoomStreamPanel,
            DespawnOnExit(Screen::Lobby),
            GlobalZIndex(STREAM_Z),
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(STREAM_TOP),
                left: Val::Px(STREAM_LEFT),
                right: Val::Px(STREAM_RIGHT),
                bottom: Val::Px(STREAM_BOTTOM),
                padding: UiRect::all(Val::Px(STREAM_FRAME_PADDING)),
                border_radius: BorderRadius::all(Val::Px(STREAM_RADIUS)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(PANEL_COLOR),
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    RoomStreamViewport,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect {
                            top: Val::Px(STREAM_IMAGE_TOP_INSET),
                            left: Val::Px(STREAM_IMAGE_PADDING),
                            right: Val::Px(STREAM_IMAGE_PADDING),
                            bottom: Val::Px(STREAM_IMAGE_PADDING),
                        },
                        border_radius: BorderRadius::all(Val::Px(STREAM_VIEWPORT_RADIUS)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(INPUT_COLOR),
                ))
                .with_children(|viewport| {
                    viewport.spawn((
                        RoomStreamImage,
                        ImageNode::default(),
                        Visibility::Hidden,
                        Node {
                            width: Val::Px(0.0),
                            height: Val::Px(0.0),
                            ..default()
                        },
                    ));
                    viewport
                        .spawn((
                            RoomStreamPlaceholder,
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|placeholder| {
                            placeholder.spawn((
                                SpinnerRing,
                                Node {
                                    width: Val::Px(STREAM_PLACEHOLDER_SPINNER_SIZE),
                                    height: Val::Px(STREAM_PLACEHOLDER_SPINNER_SIZE),
                                    border: UiRect::all(Val::Px(STREAM_PLACEHOLDER_SPINNER_WIDTH)),
                                    border_radius: BorderRadius::MAX,
                                    ..default()
                                },
                                BorderColor {
                                    top: ACCENT_COLOR,
                                    ..BorderColor::all(SPINNER_TRACK_COLOR)
                                },
                                UiTransform::default(),
                            ));
                        });
                });
            ui::discovery::bevy_systems::spawn_stream_header(panel);
        });
}

#[cfg(test)]
mod png_tests {
    use super::spawn_room_stream;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn spawn_room_stream_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_room_stream"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: |app: &mut App| {
                let row = |name: &str, occupants: usize, is_full: bool| core_discovery::LobbyRow {
                    name: core_discovery::RoomName::from(name),
                    occupants,
                    room_size: core_discovery::RoomSize::try_from(4).expect("4"),
                    occupancy: if is_full {
                        core_discovery::RoomOccupancy::Full
                    } else {
                        core_discovery::RoomOccupancy::HasSpace
                    },
                    friends: 0,
                };
                app.insert_resource(ui::discovery::StreamDisplay::default());
                app.insert_resource(ui::discovery::StreamedRoom(Some(
                    core_discovery::RoomName::from("alpha"),
                )));
                app.insert_resource(core_discovery::LobbyRows(vec![
                    row("alpha", 2, false),
                    row("gamma", 4, true),
                ]));
                app.add_systems(Startup, spawn_room_stream);
                app.add_systems(
                    Update,
                    (
                        ui::discovery::bevy_systems::refresh_room_stream,
                        ui::discovery::bevy_systems::layout_stream_panel,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("stream panel placeholder"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("fullscreen placeholder"),
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
