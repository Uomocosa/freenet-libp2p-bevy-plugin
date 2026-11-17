use crate::ui;
use bevy::prelude::*;
use ui::discovery::{
    BUTTON_LABEL_COLOR, STREAM_CLOSE_ICON_TOP, STREAM_ICON_SIDE, STREAM_ICON_STROKE,
};

pub fn spawn_close_bars(icon: &mut ChildSpawnerCommands) {
    for rotation in [std::f32::consts::FRAC_PI_4, -std::f32::consts::FRAC_PI_4] {
        icon.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(STREAM_ICON_SIDE),
                height: Val::Px(STREAM_ICON_STROKE),
                top: Val::Px(STREAM_CLOSE_ICON_TOP),
                left: Val::Px(0.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            UiTransform {
                rotation: Rot2::radians(rotation),
                ..default()
            },
            BackgroundColor(BUTTON_LABEL_COLOR),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::spawn_close_bars;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            commands
                .spawn(Node::default())
                .with_children(|icon| spawn_close_bars(&mut icon.reborrow()));
        });
        app.update();
        let mut icons = app.world_mut().query_filtered::<&Children, With<Node>>();
        let bars = icons
            .iter(app.world())
            .map(Children::len)
            .max()
            .unwrap_or_default();
        assert_eq!(bars, 2);
    }
}

#[cfg(test)]
mod png_tests {
    use super::spawn_close_bars;
    use crate::ui;
    use bevy::prelude::*;

    fn spawn_icon_preview(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands
            .spawn((
                ChildOf(root),
                Node {
                    width: Val::Px(28.0),
                    height: Val::Px(28.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(ui::discovery::BUTTON_COLOR),
            ))
            .with_children(|button| {
                button
                    .spawn(Node {
                        width: Val::Px(14.0),
                        height: Val::Px(14.0),
                        ..default()
                    })
                    .with_children(|icon| spawn_close_bars(&mut icon.reborrow()));
            });
    }

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_icon_preview);
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
    }

    #[test]
    fn spawn_close_bars_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_close_bars"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("close glyph"),
                apply: |_world: &mut World| {},
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
