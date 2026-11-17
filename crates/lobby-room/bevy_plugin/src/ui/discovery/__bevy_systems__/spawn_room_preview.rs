use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    ACCENT_COLOR, INPUT_COLOR, PANEL_COLOR, PREVIEW_FRAME_COLOR, PREVIEW_FRAME_WIDTH,
    PREVIEW_HEIGHT, PREVIEW_INNER_RADIUS, PREVIEW_PADDING, PREVIEW_PLACEHOLDER_SPINNER_SIZE,
    PREVIEW_PLACEHOLDER_SPINNER_WIDTH, PREVIEW_RADIUS, PREVIEW_RIGHT, PREVIEW_TOP, PREVIEW_WIDTH,
    PREVIEW_Z, RoomPreviewPanel, RoomPreviewPlaceholder, RoomPreviewViewport, SPINNER_TRACK_COLOR,
    Screen, SpinnerRing,
};

pub fn spawn_room_preview(mut commands: Commands) {
    commands
        .spawn((
            RoomPreviewPanel,
            DespawnOnExit(Screen::Lobby),
            GlobalZIndex(PREVIEW_Z),
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(PREVIEW_TOP),
                right: Val::Px(PREVIEW_RIGHT),
                width: Val::Px(PREVIEW_WIDTH),
                height: Val::Px(PREVIEW_HEIGHT),
                padding: UiRect::all(Val::Px(PREVIEW_PADDING)),
                border: UiRect::all(Val::Px(PREVIEW_FRAME_WIDTH)),
                border_radius: BorderRadius::all(Val::Px(PREVIEW_RADIUS)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(PANEL_COLOR),
            BorderColor::all(PREVIEW_FRAME_COLOR),
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    RoomPreviewViewport,
                    Node {
                        flex_grow: 1.0,
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(PREVIEW_INNER_RADIUS)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(INPUT_COLOR),
                ))
                .with_children(|viewport| {
                    viewport
                        .spawn((RoomPreviewPlaceholder, Node::default()))
                        .with_children(|placeholder| {
                            placeholder.spawn((
                                SpinnerRing,
                                Node {
                                    width: Val::Px(PREVIEW_PLACEHOLDER_SPINNER_SIZE),
                                    height: Val::Px(PREVIEW_PLACEHOLDER_SPINNER_SIZE),
                                    border: UiRect::all(Val::Px(PREVIEW_PLACEHOLDER_SPINNER_WIDTH)),
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
        });
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::spawn_room_preview;

    fn preview_row(name: &str, occupants: usize) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 0,
        }
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(core_discovery::LobbyRows(vec![preview_row("alpha", 2)]));
        app.insert_resource(ui::discovery::StreamedRoom::default());
        app.insert_resource(ui::discovery::SelectedRoom(Some(
            core_discovery::RoomName::from("alpha"),
        )));
        app.add_systems(Startup, spawn_room_preview);
        app.add_systems(Update, ui::discovery::bevy_systems::refresh_room_preview);
    }

    fn select_alpha(world: &mut World) {
        **world.resource_mut::<ui::discovery::SelectedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, spawn_room_preview);
        app.update();
        let mut panels = app
            .world_mut()
            .query_filtered::<(&Visibility, &Node), With<ui::discovery::RoomPreviewPanel>>();
        let (visibility, node) = panels.single(app.world()).expect("preview panel");
        assert_eq!(*visibility, Visibility::Hidden);
        assert_eq!(node.width, Val::Px(ui::discovery::PREVIEW_WIDTH));
        assert_eq!(node.height, Val::Px(ui::discovery::PREVIEW_HEIGHT));
        assert_ne!(node.border_radius, BorderRadius::ZERO);
        let mut placeholders = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomPreviewPlaceholder>>();
        assert_eq!(placeholders.iter(app.world()).count(), 1);
        let mut rings = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::SpinnerRing>>();
        assert_eq!(rings.iter(app.world()).count(), 1);
    }

    #[test]
    fn the_preview_has_no_member_names() {
        let mut app = App::new();
        app.insert_resource(core_discovery::LobbyRows(vec![preview_row("alpha", 2)]));
        app.insert_resource(ui::discovery::StreamedRoom::default());
        app.insert_resource(ui::discovery::SelectedRoom(Some(
            core_discovery::RoomName::from("alpha"),
        )));
        app.add_systems(Startup, spawn_room_preview);
        app.add_systems(Update, ui::discovery::bevy_systems::refresh_room_preview);
        app.update();
        let mut texts = app.world_mut().query::<&Text>();
        assert_eq!(
            texts.iter(app.world()).count(),
            0,
            "the preview is image-only and never prints a member name"
        );
    }

    #[test]
    fn spawn_room_preview_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_room_preview"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("hovering alpha"),
                apply: select_alpha,
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
