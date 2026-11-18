use crate::ui;
use bevy::math::{Rect, Vec2};
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use bevy::window::PrimaryWindow;
use freenet_libp2p_bevy_core::ui as core_ui;

use ui::discovery::{
    LOBBY_PANEL_RIGHT, PREVIEW_HEIGHT, PREVIEW_LIST_GAP, PREVIEW_RIGHT, PREVIEW_WIDTH, RoomButton,
    RoomPreviewPanel, RoomStreamHeader, SelectedRoom, StreamDisplay, StreamedRoom,
};

type HeaderNode<'w, 's> =
    Single<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform), With<RoomStreamHeader>>;
type PreviewNode<'w, 's> = Single<'w, 's, &'static mut Node, With<RoomPreviewPanel>>;

pub fn place_room_preview(
    selection: Res<SelectedRoom>,
    display: Res<StreamDisplay>,
    streamed: Option<Res<StreamedRoom>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    rows: Query<(&RoomButton, &ComputedNode, &UiGlobalTransform)>,
    header: Option<HeaderNode>,
    preview: Option<PreviewNode>,
) {
    let selection = selection.into_inner();
    let (Some(window), Some(preview)) = (window, preview) else {
        return;
    };
    if *display.into_inner() == StreamDisplay::Fullscreen {
        return;
    }
    let Some(name) = (**selection).clone() else {
        return;
    };
    let Some((_, row_node, row_transform)) = rows
        .iter()
        .find(|(button, _, _)| button.as_str() == name.as_str())
    else {
        return;
    };
    let anchor = core_ui::floating::PopupPlacement::rect_of(row_node, row_transform);
    let margin = LOBBY_PANEL_RIGHT;
    let mut min_top = margin;
    let is_streaming = streamed.is_some_and(|streamed| (**streamed).is_some());
    if is_streaming {
        let Some(header) = header else {
            return;
        };
        let (header_node, header_transform) = header.into_inner();
        let (_, _, header_translation) = header_transform.to_scale_angle_translation();
        let header_half = header_node.size().y * header_node.inverse_scale_factor() / 2.0;
        let header_bottom = header_translation
            .y
            .mul_add(header_node.inverse_scale_factor(), header_half);
        min_top = min_top.max(header_bottom + PREVIEW_LIST_GAP);
    }
    let area = Rect::from_corners(
        Vec2::new(margin, min_top),
        Vec2::new(window.width() - margin, window.height() - margin),
    );
    let wanted = Vec2::new(anchor.center().x, anchor.center().y - PREVIEW_HEIGHT / 2.0);
    let top = core_ui::floating::PopupPlacement::clamped_into(
        area,
        Vec2::new(PREVIEW_WIDTH, PREVIEW_HEIGHT),
        wanted,
    )
    .y;
    let mut node = preview.into_inner();
    let wanted_top = Val::Px(top);
    if node.top != wanted_top {
        node.top = wanted_top;
    }
    if node.right != Val::Px(PREVIEW_RIGHT) {
        node.right = Val::Px(PREVIEW_RIGHT);
    }
    if node.left != Val::Auto {
        node.left = Val::Auto;
    }
    if node.bottom != Val::Auto {
        node.bottom = Val::Auto;
    }
}

#[cfg(test)]
mod png_tests {
    use super::place_room_preview;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn place_room_preview_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("place_room_preview"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: |app: &mut App| {
                let row = |name: &str| core_discovery::LobbyRow {
                    name: core_discovery::RoomName::from(name),
                    occupants: 2,
                    room_size: core_discovery::RoomSize::try_from(8).expect("8"),
                    occupancy: core_discovery::RoomOccupancy::HasSpace,
                    friends: 0,
                };
                app.insert_resource(core_discovery::LobbyRows(vec![
                    row("alpha"),
                    row("beta"),
                    row("gamma"),
                ]));
                app.insert_resource(ui::discovery::SelectedRoom(Some(
                    core_discovery::RoomName::from("beta"),
                )));
                app.insert_resource(ui::discovery::StreamedRoom::default());
                app.init_resource::<ui::discovery::StreamDisplay>();
                app.add_systems(
                    Startup,
                    (
                        |mut commands: Commands| {
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
                                    for name in ["alpha", "beta", "gamma"] {
                                        let row = core_discovery::LobbyRow {
                                            name: core_discovery::RoomName::from(name),
                                            occupants: 2,
                                            room_size: core_discovery::RoomSize::try_from(8)
                                                .expect("8"),
                                            occupancy: core_discovery::RoomOccupancy::HasSpace,
                                            friends: 0,
                                        };
                                        ui::discovery::bevy_systems::spawn_room_row(list, &row);
                                    }
                                });
                        },
                        ui::discovery::bevy_systems::spawn_room_preview,
                    ),
                );
                app.add_systems(
                    Update,
                    (
                        ui::discovery::bevy_systems::refresh_room_preview,
                        place_room_preview,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("beside the middle row"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<ui::discovery::SelectedRoom>() =
                            Some(core_discovery::RoomName::from("beta"));
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("beside the last row"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<ui::discovery::SelectedRoom>() =
                            Some(core_discovery::RoomName::from("gamma"));
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
