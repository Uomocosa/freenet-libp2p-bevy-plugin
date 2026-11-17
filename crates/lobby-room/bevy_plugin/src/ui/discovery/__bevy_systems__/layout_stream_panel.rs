use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    PREVIEW_TOP, RoomPreviewPanel, RoomStreamPanel, STREAM_BOTTOM, STREAM_LEFT,
    STREAM_PREVIEW_BOTTOM, STREAM_RADIUS, STREAM_RIGHT, STREAM_TOP, StreamDisplay, StreamedRoom,
};

type StreamPanelNode<'w, 's> =
    Single<'w, 's, &'static mut Node, (With<RoomStreamPanel>, Without<RoomPreviewPanel>)>;
type PreviewPanelParts<'w, 's> = Single<
    'w,
    's,
    (&'static mut Node, &'static mut Visibility),
    (With<RoomPreviewPanel>, Without<RoomStreamPanel>),
>;

pub fn layout_stream_panel(
    streamed: Res<StreamedRoom>,
    display: Res<StreamDisplay>,
    panel: Option<StreamPanelNode>,
    preview: Option<PreviewPanelParts>,
) {
    let display = display.into_inner();
    let streamed = streamed.into_inner();
    let is_fullscreen = *display == StreamDisplay::Fullscreen;
    if let Some(panel) = panel {
        let mut node = panel.into_inner();
        let (top, left, right, bottom, radius) = if is_fullscreen {
            (0.0, 0.0, 0.0, 0.0, 0.0)
        } else {
            (
                STREAM_TOP,
                STREAM_LEFT,
                STREAM_RIGHT,
                STREAM_BOTTOM,
                STREAM_RADIUS,
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
    }
    if let Some(preview) = preview {
        let (mut node, mut visibility) = preview.into_inner();
        if is_fullscreen {
            *visibility = Visibility::Hidden;
        }
        let (top, bottom) = if (**streamed).is_some() {
            (Val::Auto, Val::Px(STREAM_PREVIEW_BOTTOM))
        } else {
            (Val::Px(PREVIEW_TOP), Val::Auto)
        };
        if node.top != top {
            node.top = top;
        }
        if node.bottom != bottom {
            node.bottom = bottom;
        }
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

    fn preview_node(app: &App, preview: Entity) -> (Node, Visibility) {
        (
            app.world()
                .get::<Node>(preview)
                .expect("preview node")
                .clone(),
            *app.world()
                .get::<Visibility>(preview)
                .expect("preview visibility"),
        )
    }

    #[test]
    fn test_usage() {
        let (mut app, panel, preview) =
            app_with(Some("alpha"), ui::discovery::StreamDisplay::Panel);
        app.update();
        let node = panel_node(&app, panel);
        assert_eq!(node.top, Val::Px(ui::discovery::STREAM_TOP));
        assert_eq!(node.right, Val::Px(ui::discovery::STREAM_RIGHT));
        let (preview, _visibility) = preview_node(&app, preview);
        assert_eq!(preview.top, Val::Auto);
        assert_eq!(
            preview.bottom,
            Val::Px(ui::discovery::STREAM_PREVIEW_BOTTOM)
        );
    }

    #[test]
    fn without_a_stream_the_preview_returns_to_the_top() {
        let (mut app, _panel, preview) = app_with(None, ui::discovery::StreamDisplay::Panel);
        app.update();
        let (preview, _visibility) = preview_node(&app, preview);
        assert_eq!(preview.top, Val::Px(ui::discovery::PREVIEW_TOP));
        assert_eq!(preview.bottom, Val::Auto);
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
        let (_preview, visibility) = preview_node(&app, preview);
        assert_eq!(visibility, Visibility::Hidden);
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

    fn preview_build(app: &mut App) {
        app.insert_resource(ui::discovery::StreamDisplay::default());
        app.insert_resource(ui::discovery::StreamedRoom(Some(
            core_discovery::RoomName::from("alpha"),
        )));
        app.insert_resource(ui::discovery::SelectedRoom(Some(
            core_discovery::RoomName::from("beta"),
        )));
        app.insert_resource(core_discovery::LobbyRows(vec![row("alpha"), row("beta")]));
        app.add_systems(
            Startup,
            (
                ui::discovery::bevy_systems::spawn_room_stream,
                ui::discovery::bevy_systems::spawn_room_preview,
            ),
        );
        app.add_systems(
            Update,
            (
                ui::discovery::bevy_systems::refresh_room_preview,
                ui::discovery::bevy_systems::refresh_room_stream,
                layout_stream_panel,
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
