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
                systems::refresh_spectator_pill,
                systems::refresh_spectator_tooltip,
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
    use crate::spectator;
    use crate::testing;
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    #[test]
    fn test_usage() {
        let mut app = testing::roster_app(testing::room_roster(
            core_discovery::OwnRoomStatus::Connected,
            1,
            8,
            Vec::new(),
        ));
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

    #[test]
    fn the_roster_bubble_follows_the_room_status() {
        for (own, count, expected_bubble, expected_dot, expected_word) in [
            (
                core_discovery::OwnRoomStatus::Disconnected,
                0,
                Visibility::Hidden,
                ui::roster::ROSTER_DISCONNECTED_COLOR,
                "Disconnected",
            ),
            (
                core_discovery::OwnRoomStatus::Disconnected,
                2,
                Visibility::Visible,
                ui::roster::ROSTER_DISCONNECTED_COLOR,
                "Disconnected",
            ),
            (
                core_discovery::OwnRoomStatus::Connected,
                2,
                Visibility::Visible,
                ui::roster::ROSTER_CONNECTED_COLOR,
                "Connected",
            ),
            (
                core_discovery::OwnRoomStatus::Connecting,
                0,
                Visibility::Visible,
                ui::roster::ROSTER_CONNECTING_COLOR,
                "Connecting",
            ),
        ] {
            let mut app = testing::roster_app(testing::room_roster(own, count, 8, Vec::new()));
            app.update();

            let mut bubbles = app
                .world_mut()
                .query_filtered::<&Visibility, With<ui::roster::RosterBubble>>();
            assert_eq!(
                *bubbles.single(app.world()).expect("the bubble"),
                expected_bubble,
                "a roster of {count} members"
            );

            let mut dots = app
                .world_mut()
                .query_filtered::<&BackgroundColor, With<ui::roster::RosterDot>>();
            assert_eq!(
                dots.single(app.world()).expect("the dot").0,
                expected_dot,
                "a roster of {count} members"
            );

            let mut words = app
                .world_mut()
                .query_filtered::<&Text, With<ui::roster::RosterDotTooltip>>();
            assert_eq!(
                **words.single(app.world()).expect("the dot tooltip"),
                *expected_word,
                "a roster of {count} members"
            );

            let mut peers = app
                .world_mut()
                .query_filtered::<&Text, With<ui::roster::RosterCountTooltip>>();
            assert_eq!(
                **peers.single(app.world()).expect("the count tooltip"),
                format!("Peers in the room: {count}/8")
            );

            let mut labels = app
                .world_mut()
                .query_filtered::<&Text, With<ui::roster::RosterCount>>();
            assert_eq!(
                **labels.single(app.world()).expect("the count"),
                count.to_string()
            );
        }
    }

    #[test]
    fn pressing_the_pill_reveals_the_member_rows() {
        let mut app = testing::roster_app(testing::room_roster(
            core_discovery::OwnRoomStatus::Connected,
            2,
            8,
            vec![
                testing::room_roster_entry(
                    "far",
                    "Far",
                    core_discovery::MemberRoomStatus::Connecting,
                    false,
                ),
                testing::room_roster_entry(
                    "me",
                    "Me",
                    core_discovery::MemberRoomStatus::Connected,
                    true,
                ),
            ],
        ));
        app.world_mut()
            .resource_mut::<ui::roster::MemberIcons>()
            .insert(
                freenet_libp2p_bevy_core::identity::PeerId::from("far"),
                Handle::default(),
            );
        app.update();

        let mut rows = app
            .world_mut()
            .query_filtered::<(), With<ui::roster::RosterRow>>();
        assert_eq!(
            rows.iter(app.world()).count(),
            2,
            "the collapsed list is rebuilt anyway"
        );
        let mut lists = app
            .world_mut()
            .query_filtered::<&Visibility, With<ui::roster::RosterList>>();
        assert_eq!(
            *lists.single(app.world()).expect("the list"),
            Visibility::Hidden
        );

        let mut icons = app.world_mut().query_filtered::<(), With<ImageNode>>();
        assert_eq!(
            icons.iter(app.world()).count(),
            1,
            "only the far peer has an icon"
        );
        let mut texts = app.world_mut().query::<&Text>();
        let labels: Vec<String> = texts
            .iter(app.world())
            .map(|text| text.as_str().to_string())
            .collect();
        assert!(labels.contains(&String::from("Far")));
        assert!(labels.contains(&String::from("Me")));
        assert!(labels.contains(&String::from("(you)")));

        let mut pills = app
            .world_mut()
            .query_filtered::<Entity, With<ui::roster::RosterPill>>();
        let pill = pills.single(app.world()).expect("the pill");
        app.world_mut().entity_mut(pill).insert(Pressed);
        app.update();
        let mut lists = app
            .world_mut()
            .query_filtered::<&Visibility, With<ui::roster::RosterList>>();
        assert_eq!(
            *lists.single(app.world()).expect("the list"),
            Visibility::Visible,
            "a pill press opens the list"
        );

        app.world_mut().entity_mut(pill).remove::<Pressed>();
        app.world_mut().entity_mut(pill).insert(Pressed);
        app.update();
        let mut lists = app
            .world_mut()
            .query_filtered::<&Visibility, With<ui::roster::RosterList>>();
        assert_eq!(
            *lists.single(app.world()).expect("the list"),
            Visibility::Hidden,
            "a second pill press closes it again"
        );

        app.world_mut().entity_mut(pill).remove::<Pressed>();
        app.update();
        let mut lists = app
            .world_mut()
            .query_filtered::<&Visibility, With<ui::roster::RosterList>>();
        assert_eq!(
            *lists.single(app.world()).expect("the list"),
            Visibility::Hidden,
            "a frame without a press leaves the list closed"
        );
    }

    #[test]
    fn the_spectator_pill_counts_the_watchers() {
        for (watching, expected_pill, expected_label) in [
            (0, Visibility::Hidden, "0 players watching"),
            (1, Visibility::Inherited, "1 player watching"),
            (12, Visibility::Inherited, "12 players watching"),
        ] {
            let mut app = testing::roster_app(testing::room_roster(
                core_discovery::OwnRoomStatus::Connected,
                3,
                8,
                Vec::new(),
            ));
            app.insert_resource(spectator::SpectatorCount(watching));
            app.update();

            let mut pills = app
                .world_mut()
                .query_filtered::<&Visibility, With<ui::roster::SpectatorPill>>();
            assert_eq!(
                *pills.single(app.world()).expect("the spectator pill"),
                expected_pill,
                "{watching} watching"
            );
            let mut counts = app
                .world_mut()
                .query_filtered::<&Text, With<ui::roster::SpectatorCountText>>();
            assert_eq!(
                **counts.single(app.world()).expect("the spectator count"),
                watching.to_string()
            );
            let mut labels = app
                .world_mut()
                .query_filtered::<&Text, With<ui::roster::SpectatorPillTooltip>>();
            assert_eq!(
                **labels.single(app.world()).expect("the spectator tooltip"),
                *expected_label
            );
        }
    }

    #[test]
    fn roster_ui_plugin_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("roster_ui_plugin"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: |app: &mut App| {
                app.add_systems(Startup, |mut commands: Commands| {
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
                });
                testing::roster_preview_app(app);
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("collapsed"),
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

    #[test]
    fn roster_ui_plugin_ui_scene_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("roster_ui_plugin"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: |app: &mut App| {
                app.add_systems(Startup, |mut commands: Commands| {
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
                });
                testing::roster_preview_app(app);
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("collapsed"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("expanded"),
                    apply: |world: &mut World| {
                        world.insert_resource(ui::roster::ListState(true));
                    },
                },
            ],
            timeline: None,
        }
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
