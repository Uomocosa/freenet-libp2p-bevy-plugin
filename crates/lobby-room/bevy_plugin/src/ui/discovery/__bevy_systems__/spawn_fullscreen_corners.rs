use crate::ui;
use bevy::prelude::*;
use ui::discovery::{BUTTON_LABEL_COLOR, STREAM_ICON_CORNER, STREAM_ICON_STROKE};

pub fn spawn_fullscreen_corners(icon: &mut ChildSpawnerCommands) {
    let corner = |node: Node| (node, BorderColor::all(BUTTON_LABEL_COLOR));
    let side = Val::Px(STREAM_ICON_CORNER);
    let stroke = Val::Px(STREAM_ICON_STROKE);
    let base = || Node {
        position_type: PositionType::Absolute,
        width: side,
        height: side,
        ..default()
    };
    icon.spawn(corner(Node {
        top: Val::Px(0.0),
        left: Val::Px(0.0),
        border: UiRect {
            top: stroke,
            left: stroke,
            ..UiRect::ZERO
        },
        ..base()
    }));
    icon.spawn(corner(Node {
        top: Val::Px(0.0),
        right: Val::Px(0.0),
        border: UiRect {
            top: stroke,
            right: stroke,
            ..UiRect::ZERO
        },
        ..base()
    }));
    icon.spawn(corner(Node {
        bottom: Val::Px(0.0),
        left: Val::Px(0.0),
        border: UiRect {
            bottom: stroke,
            left: stroke,
            ..UiRect::ZERO
        },
        ..base()
    }));
    icon.spawn(corner(Node {
        bottom: Val::Px(0.0),
        right: Val::Px(0.0),
        border: UiRect {
            bottom: stroke,
            right: stroke,
            ..UiRect::ZERO
        },
        ..base()
    }));
}

#[cfg(test)]
mod tests {
    use super::spawn_fullscreen_corners;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn spawn_fullscreen_corners_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_fullscreen_corners"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(Startup, |mut commands: Commands| {
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
                                .with_children(|icon| {
                                    spawn_fullscreen_corners(&mut icon.reborrow());
                                });
                        });
                });
                app.add_systems(
                    PostStartup,
                    lele_bevy_preview::scene::bevy_systems::center_roots,
                );
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("fullscreen glyph"),
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
