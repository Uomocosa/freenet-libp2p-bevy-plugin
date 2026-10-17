use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};

use crate::discovery;
use discovery::ui::basic::constants::{
    BUTTON_COLOR, CARET_COLOR, CARET_SELECTION_COLOR, INPUT_COLOR, PANEL_COLOR,
};
use discovery::ui::{
    CreateRoomButton, HoverGlow, HoverOutlineOnly, RoomList, RoomNameInput, Root, Screen,
};

pub fn spawn_root(mut commands: Commands) {
    commands.spawn((
        Root,
        DespawnOnExit(Screen::Lobby),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            right: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(PANEL_COLOR),
        children![
            (
                RoomNameInput,
                EditableText::new(""),
                TextCursorStyle {
                    color: CARET_COLOR,
                    selection_color: CARET_SELECTION_COLOR,
                    unfocused_selection_color: Color::NONE,
                    selected_text_color: None,
                },
                Interaction::None,
                HoverGlow::default(),
                HoverOutlineOnly,
                Node {
                    min_width: Val::Px(200.0),
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(INPUT_COLOR),
            ),
            (
                CreateRoomButton,
                Button,
                HoverGlow::default(),
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_COLOR),
                children![Text::new("+ Create New Room")],
            ),
            (
                RoomList,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
            ),
        ],
    ));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_root;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, spawn_root);
        app.update();
        let mut lists = app
            .world_mut()
            .query_filtered::<(), With<discovery::ui::RoomList>>();
        assert_eq!(lists.iter(app.world()).count(), 1);
        let mut inputs = app
            .world_mut()
            .query_filtered::<(), With<discovery::ui::RoomNameInput>>();
        assert_eq!(inputs.iter(app.world()).count(), 1);
    }

    fn preview_build(app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_root,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn spawn_root_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::run(
            &lele_bevy_preview::scene::Scene {
                name: String::from("spawn_root"),
                kind: lele_bevy_preview::scene::Kind::System,
                build: preview_build,
                states: vec![lele_bevy_preview::scene::State {
                    label: String::from("empty lobby"),
                    apply: preview_noop,
                }],
                timeline: None,
            },
            &lele_bevy_preview::preview::Config {
                out_dir: out,
                ..Default::default()
            },
            env!("CARGO_PKG_NAME"),
        )
        .expect("preview");
    }
}
