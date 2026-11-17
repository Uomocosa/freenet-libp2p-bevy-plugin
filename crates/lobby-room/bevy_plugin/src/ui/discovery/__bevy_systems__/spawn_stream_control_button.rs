use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use ui::discovery::{
    BUTTON_COLOR, HOVER_OUTLINE_WIDTH, HoverGlow, STREAM_CONTROL_RADIUS, STREAM_CONTROL_SIDE,
    STREAM_ICON_SIDE,
};

pub fn spawn_stream_control_button<M: Component>(
    header: &mut ChildSpawnerCommands,
    marker: M,
    icon: fn(&mut ChildSpawnerCommands),
) {
    header
        .spawn((
            marker,
            Button,
            Hovered::default(),
            HoverGlow::default(),
            UiTransform::default(),
            Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
            Node {
                width: Val::Px(STREAM_CONTROL_SIDE),
                height: Val::Px(STREAM_CONTROL_SIDE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(STREAM_CONTROL_RADIUS)),
                ..default()
            },
            BackgroundColor(BUTTON_COLOR),
        ))
        .with_children(|button| {
            button
                .spawn(Node {
                    width: Val::Px(STREAM_ICON_SIDE),
                    height: Val::Px(STREAM_ICON_SIDE),
                    ..default()
                })
                .with_children(icon);
        });
}

#[cfg(test)]
mod tests {
    use super::spawn_stream_control_button;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_empty().with_children(|header| {
                spawn_stream_control_button(
                    header,
                    ui::discovery::RoomStreamCloseButton,
                    ui::discovery::bevy_systems::spawn_close_bars,
                );
            });
        });
        app.update();
        let mut buttons = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomStreamCloseButton>>();
        assert_eq!(buttons.iter(app.world()).count(), 1);
    }
}

#[cfg(test)]
mod png_tests {
    use super::spawn_stream_control_button;
    use crate::ui;
    use bevy::prelude::*;

    fn spawn_controls_preview(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands
            .spawn((
                ChildOf(root),
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(ui::discovery::PANEL_COLOR),
            ))
            .with_children(|row| {
                spawn_stream_control_button(
                    row,
                    ui::discovery::RoomStreamFullscreenButton,
                    ui::discovery::bevy_systems::spawn_fullscreen_corners,
                );
                spawn_stream_control_button(
                    row,
                    ui::discovery::RoomStreamCloseButton,
                    ui::discovery::bevy_systems::spawn_close_bars,
                );
            });
    }

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_controls_preview);
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
    }

    #[test]
    fn spawn_stream_control_button_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_stream_control_button"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("fullscreen and close icons"),
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
