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

    fn app_with(
        selection: ui::discovery::SelectedRoom,
        streamed: ui::discovery::StreamedRoom,
    ) -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(selection);
        app.insert_resource(streamed);
        let outline = || {
            Outline::new(
                Val::Px(ui::discovery::HOVER_OUTLINE_WIDTH),
                Val::ZERO,
                Color::NONE,
            )
        };
        let alpha = app
            .world_mut()
            .spawn((
                ui::discovery::RoomButton(core_discovery::RoomName::from("alpha")),
                ui::discovery::HoverGlow::default(),
                UiTransform::default(),
                outline(),
                BackgroundColor(ui::basic::constants::ROOM_COLOR),
            ))
            .id();
        let beta = app
            .world_mut()
            .spawn((
                ui::discovery::RoomButton(core_discovery::RoomName::from("beta")),
                ui::discovery::HoverGlow::default(),
                UiTransform::default(),
                outline(),
                BackgroundColor(ui::basic::constants::ROOM_COLOR),
            ))
            .id();
        app.add_systems(
            Update,
            (
                ui::discovery::bevy_systems::animate_hover,
                highlight_selected_room,
            )
                .chain(),
        );
        (app, alpha, beta)
    }

    fn parts(app: &World, entity: Entity) -> (Val, Color, Color) {
        let outline = app.get::<Outline>(entity).expect("outline");
        let BackgroundColor(background) = app.get::<BackgroundColor>(entity).expect("background");
        (outline.width, outline.color, *background)
    }

    fn selected(room: &str) -> ui::discovery::SelectedRoom {
        ui::discovery::SelectedRoom(Some(core_discovery::RoomName::from(room)))
    }

    fn streamed(room: &str) -> ui::discovery::StreamedRoom {
        ui::discovery::StreamedRoom(Some(core_discovery::RoomName::from(room)))
    }

    fn nothing_streamed() -> ui::discovery::StreamedRoom {
        ui::discovery::StreamedRoom::default()
    }

    #[test]
    fn test_usage() {
        let (mut app, alpha, _beta) = app_with(selected("alpha"), streamed("alpha"));
        app.update();
        let (width, color, background) = parts(app.world(), alpha);
        assert_eq!(width, Val::Px(ui::discovery::STREAMED_ROOM_OUTLINE_WIDTH));
        assert_eq!(color, ui::discovery::STREAMED_ROOM_OUTLINE_COLOR);
        assert_eq!(background, ui::discovery::STREAMED_ROOM_BG_COLOR);
    }

    #[test]
    fn a_hover_preview_row_gets_the_soft_outline_without_a_background_tint() {
        let (mut app, alpha, _beta) = app_with(selected("alpha"), nothing_streamed());
        app.update();
        let (width, color, background) = parts(app.world(), alpha);
        assert_eq!(width, Val::Px(ui::discovery::HOVER_OUTLINE_WIDTH));
        assert_eq!(color, ui::discovery::SELECTED_ROOM_PREVIEW_OUTLINE_COLOR);
        assert_eq!(background, ui::basic::constants::ROOM_COLOR);
    }

    #[test]
    fn the_previous_row_returns_to_its_plain_look() {
        let (mut app, alpha, beta) = app_with(selected("beta"), streamed("beta"));
        app.update();
        let (width, _color, background) = parts(app.world(), beta);
        assert_eq!(width, Val::Px(ui::discovery::STREAMED_ROOM_OUTLINE_WIDTH));
        assert_eq!(background, ui::discovery::STREAMED_ROOM_BG_COLOR);
        let (alpha_width, alpha_color, alpha_background) = parts(app.world(), alpha);
        assert_eq!(alpha_width, Val::Px(ui::discovery::HOVER_OUTLINE_WIDTH));
        assert_eq!(alpha_color.alpha(), 0.0);
        assert_eq!(alpha_background, ui::basic::constants::ROOM_COLOR);
    }

    #[test]
    fn a_hovered_room_keeps_the_soft_look_while_another_room_streams() {
        let (mut app, alpha, _beta) = app_with(selected("alpha"), streamed("beta"));
        app.update();
        let (width, color, background) = parts(app.world(), alpha);
        assert_eq!(width, Val::Px(ui::discovery::HOVER_OUTLINE_WIDTH));
        assert_eq!(color, ui::discovery::SELECTED_ROOM_PREVIEW_OUTLINE_COLOR);
        assert_eq!(background, ui::basic::constants::ROOM_COLOR);
    }

    fn preview_row(name: &str) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants: 2,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 0,
        }
    }

    fn spawn_rows(mut commands: Commands) {
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
                    ui::discovery::bevy_systems::spawn_room_row(list, &preview_row(name));
                }
            });
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(ui::discovery::SelectedRoom::default());
        app.insert_resource(ui::discovery::StreamedRoom::default());
        app.add_systems(Startup, spawn_rows);
        app.add_systems(
            Update,
            (
                ui::discovery::bevy_systems::animate_hover,
                highlight_selected_room,
            )
                .chain(),
        );
    }

    fn select_alpha(world: &mut World) {
        **world.resource_mut::<ui::discovery::SelectedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
    }

    fn stream_alpha(world: &mut World) {
        **world.resource_mut::<ui::discovery::StreamedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
    }

    #[test]
    fn highlight_selected_room_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("highlight_selected_room"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("hover preview"),
                    apply: select_alpha,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("streamed room"),
                    apply: stream_alpha,
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
