use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    BUTTON_LABEL_COLOR, INPUT_COLOR, PANEL_COLOR, PREVIEW_HEIGHT, PREVIEW_MEMBER_GAP,
    PREVIEW_PADDING, PREVIEW_RIGHT, PREVIEW_ROW_GAP, PREVIEW_TITLE_FONT_SIZE, PREVIEW_TOP,
    PREVIEW_VIEWPORT_PADDING, PREVIEW_WIDTH, RoomPreviewActionSlot, RoomPreviewPanel,
    RoomPreviewTitle, RoomPreviewViewport, Screen,
};

pub fn spawn_room_preview(mut commands: Commands) {
    commands
        .spawn((
            RoomPreviewPanel,
            DespawnOnExit(Screen::Lobby),
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(PREVIEW_TOP),
                right: Val::Px(PREVIEW_RIGHT),
                width: Val::Px(PREVIEW_WIDTH),
                height: Val::Px(PREVIEW_HEIGHT),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(PREVIEW_ROW_GAP),
                padding: UiRect::all(Val::Px(PREVIEW_PADDING)),
                ..default()
            },
            BackgroundColor(PANEL_COLOR),
        ))
        .with_children(|panel| {
            panel.spawn((
                RoomPreviewTitle,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(PREVIEW_TITLE_FONT_SIZE),
                    ..default()
                },
                TextColor(BUTTON_LABEL_COLOR),
            ));
            panel.spawn((
                RoomPreviewViewport,
                Node {
                    flex_grow: 1.0,
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(PREVIEW_MEMBER_GAP),
                    padding: UiRect::all(Val::Px(PREVIEW_VIEWPORT_PADDING)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(INPUT_COLOR),
            ));
            panel.spawn((
                RoomPreviewActionSlot,
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexEnd,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ));
        });
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::spawn_room_preview;

    fn preview_row(
        name: &str,
        occupants: usize,
        occupancy: core_discovery::RoomOccupancy,
    ) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy,
            friends: 0,
        }
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(core_discovery::LobbyRows(vec![preview_row(
            "alpha",
            2,
            core_discovery::RoomOccupancy::HasSpace,
        )]));
        app.insert_resource(ui::discovery::SelectedRoom {
            room: Some(core_discovery::RoomName::from("alpha")),
            is_locked: true,
        });
        app.add_systems(Startup, spawn_room_preview);
        app.add_systems(
            Update,
            (
                ui::discovery::bevy_systems::refresh_room_preview,
                ui::discovery::bevy_systems::rebuild_room_preview_members,
                ui::discovery::bevy_systems::rebuild_room_preview_action,
            ),
        );
    }

    fn select_alpha(world: &mut World) {
        let mut selection = world.resource_mut::<ui::discovery::SelectedRoom>();
        selection.room = Some(core_discovery::RoomName::from("alpha"));
        selection.is_locked = true;
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
        let mut viewports = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomPreviewViewport>>();
        assert_eq!(viewports.iter(app.world()).count(), 1);
        let mut slots = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomPreviewActionSlot>>();
        assert_eq!(slots.iter(app.world()).count(), 1);
    }

    #[test]
    fn spawn_room_preview_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_room_preview"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("room selected"),
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
