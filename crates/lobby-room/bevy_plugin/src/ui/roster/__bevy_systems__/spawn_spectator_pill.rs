use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use ui::roster::{
    ROSTER_COUNT_FONT_SIZE, ROSTER_LABEL_COLOR, SPECTATOR_BRIDGE_HEIGHT, SPECTATOR_BRIDGE_WIDTH,
    SPECTATOR_CYAN, SPECTATOR_LENS_BORDER, SPECTATOR_LENS_GAP, SPECTATOR_LENS_SIDE,
    SpectatorCountText, SpectatorPill,
};

pub fn spawn_spectator_pill(row: &mut ChildSpawnerCommands) {
    let lens = || {
        (
            Node {
                width: Val::Px(SPECTATOR_LENS_SIDE),
                height: Val::Px(SPECTATOR_LENS_SIDE),
                border: UiRect::all(Val::Px(SPECTATOR_LENS_BORDER)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BorderColor::all(SPECTATOR_CYAN),
        )
    };
    row.spawn((
        SpectatorPill,
        Button,
        Hovered::default(),
        Visibility::Hidden,
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(ui::constants::DRAWER_PANEL_COLOR),
    ))
    .with_children(|pill| {
        pill.spawn((Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(SPECTATOR_LENS_GAP),
            ..default()
        },))
            .with_children(|lenses| {
                lenses.spawn(lens());
                lenses.spawn((
                    Node {
                        width: Val::Px(SPECTATOR_BRIDGE_WIDTH),
                        height: Val::Px(SPECTATOR_BRIDGE_HEIGHT),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(SPECTATOR_CYAN),
                ));
                lenses.spawn(lens());
            });
        pill.spawn((
            SpectatorCountText,
            Hovered::default(),
            Text::new("0"),
            TextFont {
                font_size: FontSize::Px(ROSTER_COUNT_FONT_SIZE),
                ..default()
            },
            TextColor(ROSTER_LABEL_COLOR),
        ));
    });
}

#[cfg(test)]
mod tests {
    use super::spawn_spectator_pill;
    use crate::spectator;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            commands
                .spawn((Node::default(),))
                .with_children(spawn_spectator_pill);
        });
        app.update();
        let mut pills = app
            .world_mut()
            .query_filtered::<(), With<ui::roster::SpectatorPill>>();
        assert_eq!(pills.iter(app.world()).count(), 1);
        let mut counts = app
            .world_mut()
            .query_filtered::<(), With<ui::roster::SpectatorCountText>>();
        assert_eq!(counts.iter(app.world()).count(), 1);
    }

    fn spawn_mock_screen(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands
            .spawn((
                ChildOf(root),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(ui::constants::ROOM_COLOR),
            ))
            .with_children(|screen| {
                screen
                    .spawn((Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(16.0),
                        left: Val::Px(16.0),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    },))
                    .with_children(|row| {
                        row.spawn((
                            Node {
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(7.0),
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(ui::constants::DRAWER_PANEL_COLOR),
                        ))
                        .with_children(|pill| {
                            pill.spawn((
                                Text::new("3"),
                                TextFont {
                                    font_size: FontSize::Px(ui::roster::ROSTER_COUNT_FONT_SIZE),
                                    ..default()
                                },
                                TextColor(ui::roster::ROSTER_LABEL_COLOR),
                            ));
                        });
                        spawn_spectator_pill(row);
                    });
            });
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(spectator::SpectatorCount::default());
        app.add_systems(Startup, spawn_mock_screen);
        app.add_systems(Update, ui::roster::bevy_systems::refresh_spectator_pill);
    }

    fn two_watching(world: &mut World) {
        world.insert_resource(spectator::SpectatorCount(2));
    }

    #[test]
    fn spawn_spectator_pill_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_spectator_pill"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("two watching"),
                apply: two_watching,
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
