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
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::layout_stream_panel;

    fn app_with(
        streamed: Option<&str>,
        display: ui::discovery::StreamDisplay,
    ) -> (App, Entity, Entity) {
        let mut app = App::new();
        app.insert_resource(ui::discovery::StreamedRoom(
            streamed.map(core_discovery::RoomName::from),
        ));
        app.insert_resource(display);
        let panel = app
            .world_mut()
            .spawn((ui::discovery::RoomStreamPanel, Node::default()))
            .id();
        let preview = app
            .world_mut()
            .spawn((
                ui::discovery::RoomPreviewPanel,
                Node {
                    top: Val::Px(ui::discovery::PREVIEW_TOP),
                    ..default()
                },
                Visibility::Hidden,
            ))
            .id();
        app.add_systems(Update, layout_stream_panel);
        (app, panel, preview)
    }

    fn panel_node(app: &App, panel: Entity) -> Node {
        app.world().get::<Node>(panel).expect("panel node").clone()
    }

    fn preview_visibility(app: &App, preview: Entity) -> Visibility {
        *app.world()
            .get::<Visibility>(preview)
            .expect("preview visibility")
    }

    #[test]
    fn test_usage() {
        let (mut app, panel, _preview) =
            app_with(Some("alpha"), ui::discovery::StreamDisplay::Panel);
        app.update();
        let node = panel_node(&app, panel);
        assert_eq!(node.top, Val::Px(ui::discovery::STREAM_TOP));
        assert_eq!(node.right, Val::Px(ui::discovery::STREAM_RIGHT));
        assert_eq!(
            node.padding,
            UiRect::all(Val::Px(ui::discovery::STREAM_FRAME_PADDING))
        );
    }

    #[test]
    fn fullscreen_fills_the_window_and_hides_the_preview() {
        let (mut app, panel, preview) =
            app_with(Some("alpha"), ui::discovery::StreamDisplay::Fullscreen);
        app.world_mut()
            .entity_mut(preview)
            .insert(Visibility::Visible);
        app.update();
        let node = panel_node(&app, panel);
        assert_eq!(node.top, Val::Px(0.0));
        assert_eq!(node.left, Val::Px(0.0));
        assert_eq!(node.right, Val::Px(0.0));
        assert_eq!(node.bottom, Val::Px(0.0));
        assert_eq!(node.padding, UiRect::ZERO);
        assert_eq!(preview_visibility(&app, preview), Visibility::Hidden);
    }
}

#[cfg(test)]
mod png_tests {
    use super::layout_stream_panel;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    fn row(name: &str) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants: 2,
            room_size: core_discovery::RoomSize::try_from(4).expect("4"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 0,
        }
    }

    fn spawn_list(mut commands: Commands) {
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
                    ui::discovery::bevy_systems::spawn_room_row(list, &row(name));
                }
            });
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(ui::discovery::StreamDisplay::default());
        app.insert_resource(ui::discovery::StreamedRoom(Some(
            core_discovery::RoomName::from("alpha"),
        )));
        app.insert_resource(ui::discovery::SelectedRoom(Some(
            core_discovery::RoomName::from("beta"),
        )));
        app.insert_resource(core_discovery::LobbyRows(vec![
            row("alpha"),
            row("beta"),
            row("gamma"),
        ]));
        app.add_systems(
            Startup,
            (
                ui::discovery::bevy_systems::spawn_room_stream,
                ui::discovery::bevy_systems::spawn_room_preview,
                spawn_list,
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
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_fullscreen(world: &mut World) {
        *world.resource_mut::<ui::discovery::StreamDisplay>() =
            ui::discovery::StreamDisplay::Fullscreen;
    }

    #[test]
    fn layout_stream_panel_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("layout_stream_panel"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("panel with preview"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("fullscreen"),
                    apply: preview_fullscreen,
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
