use bevy::prelude::*;

use crate::discovery;
use discovery::ui::basic::constants::{BUTTON_COLOR, PANEL_COLOR};
use discovery::ui::{JoinError, JoinErrorDismissButton, JoinErrorRoot, Screen};

pub fn spawn_join_failed(mut commands: Commands, join_error: Res<JoinError>) {
    let join_error = join_error.into_inner();
    let message = join_error
        .as_deref()
        .unwrap_or("Couldn't get room history")
        .to_string();
    commands.spawn((
        JoinErrorRoot,
        DespawnOnExit(Screen::JoinFailed),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(18.0)),
                ..default()
            },
            BackgroundColor(PANEL_COLOR),
            children![
                (Text::new(message), TextColor(Color::WHITE)),
                (
                    JoinErrorDismissButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(BUTTON_COLOR),
                    children![Text::new("Back to lobby")],
                ),
            ],
        )],
    ));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_join_failed;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(discovery::ui::JoinError(Some(String::from(
            "Room alpha no longer exists (all players left)",
        ))));
        app.add_systems(Startup, spawn_join_failed);
        app.update();
        let mut roots = app
            .world_mut()
            .query_filtered::<(), With<discovery::ui::JoinErrorRoot>>();
        assert_eq!(roots.iter(app.world()).count(), 1);
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(discovery::ui::JoinError(Some(String::from(
            "Room alpha no longer exists (all players left)",
        ))));
        app.add_systems(
            Startup,
            (
                spawn_join_failed,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn spawn_join_failed_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::run(
            &lele_bevy_preview::scene::Scene {
                name: String::from("spawn_join_failed"),
                kind: lele_bevy_preview::scene::Kind::System,
                build: preview_build,
                states: vec![lele_bevy_preview::scene::State {
                    label: String::from("join failed"),
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
