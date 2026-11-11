use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::direct_dial;
use crate::discovery;
use direct_dial::{DRAWER_TITLE_FONT_SIZE, FriendsDrawerCloseButton};
use discovery::{BUTTON_COLOR, BUTTON_LABEL_COLOR, HOVER_OUTLINE_WIDTH, HoverGlow};

pub fn spawn_drawer_header(header: &mut ChildSpawnerCommands) {
    header
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|row| {
            row.spawn((
                Text::new("Friends"),
                TextFont {
                    font_size: FontSize::Px(DRAWER_TITLE_FONT_SIZE),
                    ..default()
                },
                TextColor(BUTTON_LABEL_COLOR),
            ));
            row.spawn((Node {
                flex_grow: 1.0,
                ..default()
            },));
            row.spawn((
                FriendsDrawerCloseButton,
                Button,
                Hovered::default(),
                HoverGlow::default(),
                UiTransform::default(),
                Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_COLOR),
                children![Text::new("x")],
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::spawn_drawer_header;
    use bevy::prelude::*;

    use crate::basic::constants::DRAWER_PANEL_COLOR;
    use crate::direct_dial;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_empty().with_children(|panel| {
                spawn_drawer_header(panel);
            });
        });
        app.update();
        let mut close = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::FriendsDrawerCloseButton>>();
        assert_eq!(close.iter(app.world()).count(), 1);
    }

    fn spawn_header(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands
            .spawn((
                ChildOf(root),
                Node {
                    width: Val::Px(240.0),
                    ..default()
                },
                BackgroundColor(DRAWER_PANEL_COLOR),
            ))
            .with_children(|panel| {
                spawn_drawer_header(panel);
            });
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_build(app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_header,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
    }

    #[test]
    fn spawn_drawer_header_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_drawer_header"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("header row"),
                apply: preview_noop,
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
