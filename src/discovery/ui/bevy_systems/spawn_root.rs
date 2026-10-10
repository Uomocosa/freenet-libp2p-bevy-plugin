use bevy::prelude::*;
use bevy::text::EditableText;

use crate::discovery;
use discovery::ui::{CreateRoomButton, RoomList, RoomNameInput, UiRoot};

const BUTTON_COLOR: Color = Color::srgb(0.20, 0.24, 0.32);
const INPUT_COLOR: Color = Color::srgb(0.10, 0.11, 0.14);

pub fn spawn_root(mut commands: Commands) {
    commands.spawn((
        UiRoot,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        },
        children![
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                children![
                    (
                        RoomNameInput,
                        EditableText::new(""),
                        Node {
                            width: Val::Px(220.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(INPUT_COLOR),
                    ),
                    (
                        CreateRoomButton,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(BUTTON_COLOR),
                        children![Text::new("Create room")],
                    ),
                ],
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
        app.add_systems(Startup, spawn_root);
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    #[ignore = "headed preview"]
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
