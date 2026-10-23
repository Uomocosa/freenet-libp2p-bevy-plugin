use bevy::prelude::*;

use crate::discovery;
use discovery::ui::basic::constants::{
    ACCENT_COLOR, CONNECTING_TEXT_COLOR, SPINNER_SIZE, SPINNER_TRACK_COLOR,
};
use discovery::ui::{ConnectingRoot, Screen, SpinnerRing};

pub fn spawn_connecting(mut commands: Commands) {
    commands.spawn((
        ConnectingRoot,
        DespawnOnExit(Screen::Connecting),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(16.0),
            ..default()
        },
        children![
            (
                SpinnerRing,
                Node {
                    width: Val::Px(SPINNER_SIZE),
                    height: Val::Px(SPINNER_SIZE),
                    border: UiRect::all(Val::Px(4.0)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BorderColor {
                    top: ACCENT_COLOR,
                    ..BorderColor::all(SPINNER_TRACK_COLOR)
                },
                UiTransform::default(),
            ),
            (
                Text::new("Connecting to network..."),
                TextColor(CONNECTING_TEXT_COLOR),
            ),
        ],
    ));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_connecting;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, spawn_connecting);
        app.update();
        let mut rings = app
            .world_mut()
            .query_filtered::<(), With<discovery::ui::SpinnerRing>>();
        assert_eq!(rings.iter(app.world()).count(), 1);
    }

    fn preview_build(app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_connecting,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn spawn_connecting_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_connecting"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("connecting"),
                apply: preview_noop,
            }],
            timeline: None,
        }
        .run(
            &lele_bevy_preview::preview::Config {
                out_dir: out,
                ..Default::default()
            },
            env!("CARGO_PKG_NAME"),
        )
        .expect("preview");
    }
}
