use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    ACCENT_COLOR, PANEL_COLOR, RoomStreamImage, RoomStreamPanel, RoomStreamPlaceholder,
    RoomStreamViewport, SPINNER_TRACK_COLOR, STREAM_BOTTOM, STREAM_LEFT,
    STREAM_PLACEHOLDER_SPINNER_SIZE, STREAM_PLACEHOLDER_SPINNER_WIDTH, STREAM_RADIUS, STREAM_RIGHT,
    STREAM_TOP, STREAM_Z, Screen, SpinnerRing,
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
                        border_radius: BorderRadius::all(Val::Px(STREAM_RADIUS)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(PANEL_COLOR),
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
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::spawn_room_stream;

    fn row(name: &str, occupants: usize, is_full: bool) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants,
            room_size: core_discovery::RoomSize::try_from(4).expect("4"),
            occupancy: if is_full {
                core_discovery::RoomOccupancy::Full
            } else {
                core_discovery::RoomOccupancy::HasSpace
            },
            friends: 0,
        }
    }

    fn app_with(streamed: Option<&str>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ui::discovery::StreamDisplay>();
        app.add_message::<ui::discovery::CloseStream>();
        app.insert_resource(ui::discovery::StreamedRoom(
            streamed.map(core_discovery::RoomName::from),
        ));
        app.insert_resource(core_discovery::LobbyRows(vec![row("alpha", 2, false)]));
        app.add_systems(Startup, spawn_room_stream);
        app.update();
        app
    }

    fn count<C: Component>(app: &mut App) -> usize {
        let mut query = app.world_mut().query_filtered::<(), With<C>>();
        query.iter(app.world()).count()
    }

    fn press<C: Component>(app: &mut App) {
        let mut buttons = app.world_mut().query_filtered::<Entity, With<C>>();
        let button = buttons.single(app.world()).expect("button");
        app.world_mut().entity_mut(button).insert(Pressed);
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(Some("alpha"));
        assert_eq!(count::<ui::discovery::RoomStreamPanel>(&mut app), 1);
        assert_eq!(count::<ui::discovery::RoomStreamViewport>(&mut app), 1);
        assert_eq!(count::<ui::discovery::RoomStreamImage>(&mut app), 1);
        assert_eq!(count::<ui::discovery::RoomStreamPlaceholder>(&mut app), 1);
        assert_eq!(count::<ui::discovery::RoomStreamActionSlot>(&mut app), 1);
        assert_eq!(count::<ui::discovery::RoomStreamCloseButton>(&mut app), 1);
        assert_eq!(
            count::<ui::discovery::RoomStreamFullscreenButton>(&mut app),
            1
        );
        let mut panels = app
            .world_mut()
            .query_filtered::<(&Node, &Visibility), With<ui::discovery::RoomStreamPanel>>();
        let (node, visibility) = panels.single(app.world()).expect("stream panel");
        assert_eq!(*visibility, Visibility::Hidden);
        assert_eq!(node.left, Val::Px(ui::discovery::STREAM_LEFT));
        assert_eq!(node.right, Val::Px(ui::discovery::STREAM_RIGHT));
        assert_eq!(node.top, Val::Px(ui::discovery::STREAM_TOP));
        assert_eq!(node.bottom, Val::Px(ui::discovery::STREAM_BOTTOM));
    }

    #[test]
    fn the_close_button_sends_close_stream() {
        let mut app = app_with(Some("alpha"));
        app.add_systems(Update, ui::discovery::bevy_systems::close_stream_on_press);
        press::<ui::discovery::RoomStreamCloseButton>(&mut app);
        app.update();
        let messages = app
            .world()
            .resource::<Messages<ui::discovery::CloseStream>>();
        let mut cursor = messages.get_cursor();
        assert_eq!(cursor.read(messages).count(), 1);
    }

    #[test]
    fn the_fullscreen_button_toggles_the_display() {
        let mut app = app_with(Some("alpha"));
        app.add_systems(
            Update,
            ui::discovery::bevy_systems::toggle_stream_fullscreen_on_press,
        );
        press::<ui::discovery::RoomStreamFullscreenButton>(&mut app);
        app.update();
        assert_eq!(
            *app.world().resource::<ui::discovery::StreamDisplay>(),
            ui::discovery::StreamDisplay::Fullscreen
        );
    }

    fn preview_build(app: &mut App) {
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
    }

    const fn preview_noop(_world: &mut World) {}

    fn go_fullscreen(world: &mut World) {
        *world.resource_mut::<ui::discovery::StreamDisplay>() =
            ui::discovery::StreamDisplay::Fullscreen;
    }

    #[test]
    fn spawn_room_stream_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_room_stream"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("stream panel placeholder"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("fullscreen placeholder"),
                    apply: go_fullscreen,
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
