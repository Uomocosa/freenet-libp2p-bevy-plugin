use crate::ui;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use ui::discovery::{
    LOBBY_PANEL_RIGHT, PREVIEW_HEIGHT, PREVIEW_LIST_GAP, PREVIEW_RIGHT, RoomButton,
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
    let (_, _, row_translation) = row_transform.to_scale_angle_translation();
    let row_centre = row_translation.y * row_node.inverse_scale_factor();
    let margin = LOBBY_PANEL_RIGHT;
    let min_top = margin;
    let max_top = window.height() - PREVIEW_HEIGHT - margin;
    let mut top = (row_centre - PREVIEW_HEIGHT / 2.0).max(min_top);
    if max_top > min_top {
        top = top.min(max_top);
    }
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
        let clearance = header_bottom + PREVIEW_LIST_GAP;
        if top < clearance {
            top = clearance;
        }
    }
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
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::{ComputedNode, UiGlobalTransform};
    use bevy::window::{PrimaryWindow, WindowResolution};

    use super::place_room_preview;

    fn app_with(selected: Option<&str>, window_height: u32) -> (App, Entity) {
        let mut app = App::new();
        app.insert_resource(ui::discovery::SelectedRoom(
            selected.map(core_discovery::RoomName::from),
        ));
        app.insert_resource(ui::discovery::StreamDisplay::default());
        app.world_mut().spawn((
            Window {
                resolution: WindowResolution::new(960, window_height),
                ..default()
            },
            PrimaryWindow,
        ));
        app.world_mut().spawn((
            ui::discovery::RoomButton(core_discovery::RoomName::from("alpha")),
            ComputedNode {
                size: Vec2::new(232.0, 32.0),
                ..ComputedNode::default()
            },
            UiGlobalTransform::from_xy(860.0, 300.0),
        ));
        let preview = app
            .world_mut()
            .spawn((ui::discovery::RoomPreviewPanel, Node::default()))
            .id();
        app.add_systems(Update, place_room_preview);
        (app, preview)
    }

    fn preview_node(app: &App, preview: Entity) -> Node {
        app.world()
            .get::<Node>(preview)
            .expect("preview node")
            .clone()
    }

    fn move_row(app: &mut App, y: f32) {
        let mut rows = app.world_mut().query::<&mut UiGlobalTransform>();
        for mut transform in rows.iter_mut(app.world_mut()) {
            *transform = UiGlobalTransform::from_xy(860.0, y);
        }
    }

    #[test]
    fn test_usage() {
        let (mut app, preview) = app_with(Some("alpha"), 540);
        app.update();
        let node = preview_node(&app, preview);
        assert_eq!(node.top, Val::Px(200.0));
        assert_eq!(node.right, Val::Px(ui::discovery::PREVIEW_RIGHT));
        assert_eq!(node.left, Val::Auto);
        assert_eq!(node.bottom, Val::Auto);
    }

    #[test]
    fn a_top_row_clamps_to_the_window_margin() {
        let (mut app, preview) = app_with(Some("alpha"), 540);
        move_row(&mut app, 90.0);
        app.update();
        assert_eq!(
            preview_node(&app, preview).top,
            Val::Px(ui::discovery::LOBBY_PANEL_RIGHT)
        );
    }

    #[test]
    fn a_bottom_row_clamps_to_the_window_margin() {
        let (mut app, preview) = app_with(Some("alpha"), 540);
        move_row(&mut app, 520.0);
        app.update();
        assert_eq!(
            preview_node(&app, preview).top,
            Val::Px(540.0 - ui::discovery::PREVIEW_HEIGHT - ui::discovery::LOBBY_PANEL_RIGHT)
        );
    }

    #[test]
    fn the_stream_header_pushes_the_card_below_it() {
        let (mut app, preview) = app_with(Some("alpha"), 540);
        move_row(&mut app, 90.0);
        app.insert_resource(ui::discovery::StreamedRoom(Some(
            core_discovery::RoomName::from("alpha"),
        )));
        app.world_mut().spawn((
            ui::discovery::RoomStreamHeader,
            ComputedNode {
                size: Vec2::new(600.0, 40.0),
                ..ComputedNode::default()
            },
            UiGlobalTransform::from_xy(400.0, 80.0),
        ));
        app.update();
        assert_eq!(preview_node(&app, preview).top, Val::Px(116.0));
    }

    #[test]
    fn without_a_selection_the_card_is_left_alone() {
        let (mut app, preview) = app_with(None, 540);
        app.update();
        assert_eq!(preview_node(&app, preview).top, Val::Auto);
    }

    #[test]
    fn fullscreen_does_not_move_the_card() {
        let (mut app, preview) = app_with(Some("alpha"), 540);
        *app.world_mut()
            .resource_mut::<ui::discovery::StreamDisplay>() =
            ui::discovery::StreamDisplay::Fullscreen;
        app.update();
        assert_eq!(preview_node(&app, preview).top, Val::Auto);
    }
}

#[cfg(test)]
mod png_tests {
    use super::place_room_preview;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    fn row(name: &str) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants: 2,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
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
            (spawn_list, ui::discovery::bevy_systems::spawn_room_preview),
        );
        app.add_systems(
            Update,
            (
                ui::discovery::bevy_systems::refresh_room_preview,
                place_room_preview,
            )
                .chain(),
        );
    }

    fn hover_beta(world: &mut World) {
        **world.resource_mut::<ui::discovery::SelectedRoom>() =
            Some(core_discovery::RoomName::from("beta"));
    }

    fn hover_gamma(world: &mut World) {
        **world.resource_mut::<ui::discovery::SelectedRoom>() =
            Some(core_discovery::RoomName::from("gamma"));
    }

    #[test]
    fn place_room_preview_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("place_room_preview"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("beside the middle row"),
                    apply: hover_beta,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("beside the last row"),
                    apply: hover_gamma,
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
