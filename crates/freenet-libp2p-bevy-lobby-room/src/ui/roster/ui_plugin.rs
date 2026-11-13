use crate::ui;
use bevy::prelude::App;
use bevy::prelude::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct UiPlugin;

impl bevy::prelude::Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        use ui::roster::bevy_systems as systems;
        app.init_resource::<ui::roster::MemberIcons>();
        app.init_resource::<ui::roster::ListState>();
        app.add_systems(Startup, systems::spawn_roster_bubble);
        app.add_systems(
            Update,
            (
                systems::prune_member_icons,
                systems::refresh_roster_status,
                systems::refresh_roster_tooltips_text,
                systems::refresh_roster_tooltip,
                systems::toggle_roster_list,
                systems::refresh_roster_list,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::UiPlugin;

    fn app_with_roster(own: core_discovery::OwnRoomStatus, count: usize) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(core_discovery::RoomRoster {
            own,
            count,
            max: 8,
            entries: Vec::new(),
        });
        app.add_plugins(UiPlugin);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with_roster(core_discovery::OwnRoomStatus::Connected, 1);
        app.update();
        let mut bubbles = app
            .world_mut()
            .query_filtered::<(), With<ui::roster::RosterBubble>>();
        assert_eq!(bubbles.iter(app.world()).count(), 1);
        let mut pills = app
            .world_mut()
            .query_filtered::<(), With<ui::roster::RosterPill>>();
        assert_eq!(pills.iter(app.world()).count(), 1);
        let mut icons = app
            .world_mut()
            .query_filtered::<&ui::roster::MemberIcons, ()>();
        assert_eq!(icons.iter(app.world()).count(), 1);
    }

    fn preview_roster() -> core_discovery::RoomRoster {
        core_discovery::RoomRoster {
            own: core_discovery::OwnRoomStatus::Connected,
            count: 3,
            max: 8,
            entries: vec![
                entry(
                    "12D3KooWcarol0Zm9c",
                    "carol",
                    core_discovery::MemberRoomStatus::Connecting,
                    false,
                ),
                entry(
                    "12D3KooWbob000Kd2a",
                    "bob",
                    core_discovery::MemberRoomStatus::Connected,
                    false,
                ),
                entry(
                    "me",
                    "alice",
                    core_discovery::MemberRoomStatus::Connected,
                    true,
                ),
            ],
        }
    }

    fn entry(
        id: &str,
        name: &str,
        status: core_discovery::MemberRoomStatus,
        is_me: bool,
    ) -> core_discovery::RoomRosterEntry {
        core_discovery::RoomRosterEntry {
            id: freenet_libp2p_bevy_core::identity::PeerId::from(id),
            name: String::from(name),
            status,
            is_me,
        }
    }

    fn spawn_mock_screen(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(ui::constants::ROOM_COLOR),
            children![(
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(16.0),
                    right: Val::Px(16.0),
                    width: Val::Px(44.0),
                    height: Val::Px(68.0),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.47, 0.29, 0.16)),
            )],
        ));
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_roster());
        app.add_systems(Startup, spawn_mock_screen);
        app.add_plugins(UiPlugin);
    }

    const fn preview_noop(_world: &mut World) {}

    fn expand(world: &mut World) {
        world.insert_resource(ui::roster::ListState(true));
    }

    fn scene(states: Vec<lele_bevy_preview::scene::State>) -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("roster_ui_plugin"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states,
            timeline: None,
        }
    }

    #[test]
    fn roster_ui_plugin_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        scene(vec![lele_bevy_preview::scene::State {
            label: String::from("collapsed"),
            apply: preview_noop,
        }])
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }

    #[test]
    fn roster_ui_plugin_ui_scene_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        scene(vec![
            lele_bevy_preview::scene::State {
                label: String::from("collapsed"),
                apply: preview_noop,
            },
            lele_bevy_preview::scene::State {
                label: String::from("expanded"),
                apply: expand,
            },
        ])
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}

#[cfg(test)]
mod build_plugin_tests {
    use crate::ui;
    use bevy::prelude::Plugin;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        Plugin::build(&ui::roster::UiPlugin, &mut app);
        app.update();
        let mut bubbles = app
            .world_mut()
            .query_filtered::<(), With<ui::roster::RosterBubble>>();
        assert_eq!(bubbles.iter(app.world()).count(), 1);
    }
}
