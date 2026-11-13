use bevy::prelude::*;

use crate::roster;
use freenet_libp2p_bevy_lobby_room::discovery as core_discovery;
use roster::{
    ROSTER_CONNECTED_COLOR, ROSTER_CONNECTING_COLOR, ROSTER_DISCONNECTED_COLOR, RosterBubble,
    RosterCount, RosterDot,
};

pub fn refresh_roster_status(
    roster_data: Option<Res<core_discovery::RoomRoster>>,
    root: Single<&mut Visibility, With<RosterBubble>>,
    dot: Single<&mut BackgroundColor, With<RosterDot>>,
    count: Single<&mut Text, With<RosterCount>>,
) {
    let empty = core_discovery::RoomRoster::default();
    let roster = roster_data.map_or(&empty, Res::into_inner);
    let is_shown =
        !matches!(roster.own, core_discovery::OwnRoomStatus::Disconnected) || roster.count > 0;
    let wanted_visibility = if is_shown {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let mut root = root.into_inner();
    if *root != wanted_visibility {
        *root = wanted_visibility;
    }
    let mut dot = dot.into_inner();
    let wanted_dot = match roster.own {
        core_discovery::OwnRoomStatus::Connected => ROSTER_CONNECTED_COLOR,
        core_discovery::OwnRoomStatus::Connecting => ROSTER_CONNECTING_COLOR,
        core_discovery::OwnRoomStatus::Disconnected => ROSTER_DISCONNECTED_COLOR,
    };
    let BackgroundColor(current_dot) = &mut *dot;
    if *current_dot != wanted_dot {
        *current_dot = wanted_dot;
    }
    let wanted_count = roster.count.to_string();
    let mut count = count.into_inner();
    if **count != wanted_count {
        **count = wanted_count;
    }
}

#[cfg(test)]
mod tests {
    use super::refresh_roster_status;
    use crate::roster;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_lobby_room::discovery as core_discovery;

    use crate::basic::constants::ROOM_COLOR;

    fn roster_with(
        own: core_discovery::OwnRoomStatus,
        count: usize,
        max: usize,
    ) -> core_discovery::RoomRoster {
        core_discovery::RoomRoster {
            own,
            count,
            max,
            entries: Vec::new(),
        }
    }

    fn app_with(roster: core_discovery::RoomRoster) -> (App, Entity, Entity, Entity) {
        let mut app = App::new();
        app.insert_resource(roster);
        let root = app
            .world_mut()
            .spawn((roster::RosterBubble, Visibility::Hidden))
            .id();
        let dot = app
            .world_mut()
            .spawn((ChildOf(root), roster::RosterDot, BackgroundColor::default()))
            .id();
        let count = app
            .world_mut()
            .spawn((ChildOf(root), roster::RosterCount, Text::new("")))
            .id();
        app.add_systems(Update, refresh_roster_status);
        (app, root, dot, count)
    }

    #[test]
    fn test_usage() {
        let (mut app, root, _dot, count) =
            app_with(roster_with(core_discovery::OwnRoomStatus::Connected, 3, 8));
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("root"),
            Visibility::Visible
        );
        let Text(count) = app.world().get::<Text>(count).expect("count").clone();
        assert_eq!(count, String::from("3"));
    }

    #[test]
    fn an_empty_roster_hides_the_bubble() {
        let (mut app, root, _dot, _count) = app_with(core_discovery::RoomRoster::default());
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("root"),
            Visibility::Hidden
        );
    }

    #[test]
    fn the_dot_colours_follow_the_own_status() {
        let cases = [
            (
                core_discovery::OwnRoomStatus::Connected,
                roster::ROSTER_CONNECTED_COLOR,
            ),
            (
                core_discovery::OwnRoomStatus::Connecting,
                roster::ROSTER_CONNECTING_COLOR,
            ),
            (
                core_discovery::OwnRoomStatus::Disconnected,
                roster::ROSTER_DISCONNECTED_COLOR,
            ),
        ];
        for (own, wanted) in cases {
            let (mut app, _root, dot, _count) = app_with(roster_with(own, 1, 8));
            app.update();
            let BackgroundColor(colour) = *app.world().get::<BackgroundColor>(dot).expect("dot");
            assert_eq!(colour, wanted);
        }
    }

    fn refresh_connected(world: &mut World) {
        *world.resource_mut::<core_discovery::RoomRoster>() =
            roster_with(core_discovery::OwnRoomStatus::Connected, 3, 8);
    }

    fn refresh_connecting(world: &mut World) {
        *world.resource_mut::<core_discovery::RoomRoster>() =
            roster_with(core_discovery::OwnRoomStatus::Connecting, 2, 8);
    }

    fn refresh_swarm_lost(world: &mut World) {
        *world.resource_mut::<core_discovery::RoomRoster>() =
            roster_with(core_discovery::OwnRoomStatus::Disconnected, 3, 8);
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
            BackgroundColor(ROOM_COLOR),
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
                children![Text::new("leave")],
            )],
        ));
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(core_discovery::RoomRoster::default());
        app.add_systems(
            Startup,
            (spawn_mock_screen, roster::bevy_systems::spawn_roster_bubble),
        );
        app.add_systems(
            Update,
            (
                refresh_roster_status,
                roster::bevy_systems::refresh_roster_tooltips_text,
            ),
        );
    }

    #[test]
    fn refresh_roster_status_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_roster_status"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("connecting"),
                    apply: refresh_connecting,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("connected"),
                    apply: refresh_connected,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("swarm lost"),
                    apply: refresh_swarm_lost,
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
