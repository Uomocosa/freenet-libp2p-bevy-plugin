use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::roster::{RosterCountTooltip, RosterDotTooltip};

pub fn refresh_roster_tooltips_text(
    roster_data: Option<Res<core_discovery::RoomRoster>>,
    dot_tip: Single<&mut Text, (With<RosterDotTooltip>, Without<RosterCountTooltip>)>,
    count_tip: Single<&mut Text, (With<RosterCountTooltip>, Without<RosterDotTooltip>)>,
) {
    let empty = core_discovery::RoomRoster::default();
    let roster = roster_data.map_or(&empty, Res::into_inner);
    let wanted_dot_tip = String::from(match roster.own {
        core_discovery::OwnRoomStatus::Connected => "Connected",
        core_discovery::OwnRoomStatus::Connecting => "Connecting",
        core_discovery::OwnRoomStatus::Disconnected => "Disconnected",
    });
    let mut dot_tip = dot_tip.into_inner();
    if **dot_tip != wanted_dot_tip {
        **dot_tip = wanted_dot_tip;
    }
    let wanted_count_tip = format!("Peers in the room: {}/{}", roster.count, roster.max);
    let mut count_tip = count_tip.into_inner();
    if **count_tip != wanted_count_tip {
        **count_tip = wanted_count_tip;
    }
}

#[cfg(test)]
mod tests {
    use super::refresh_roster_tooltips_text;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

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

    fn app_with_tips(roster: core_discovery::RoomRoster) -> (App, Entity, Entity) {
        let mut app = App::new();
        app.insert_resource(roster);
        let dot_tip = app
            .world_mut()
            .spawn((ui::roster::RosterDotTooltip, Text::new("")))
            .id();
        let count_tip = app
            .world_mut()
            .spawn((ui::roster::RosterCountTooltip, Text::new("")))
            .id();
        app.add_systems(Update, refresh_roster_tooltips_text);
        (app, dot_tip, count_tip)
    }

    #[test]
    fn test_usage() {
        let (mut app, dot_tip, count_tip) =
            app_with_tips(roster_with(core_discovery::OwnRoomStatus::Connecting, 2, 8));
        app.update();
        let Text(word) = app.world().get::<Text>(dot_tip).expect("dot tip").clone();
        assert_eq!(word, String::from("Connecting"));
        let Text(room) = app
            .world()
            .get::<Text>(count_tip)
            .expect("count tip")
            .clone();
        assert_eq!(room, String::from("Peers in the room: 2/8"));
    }

    fn refresh_connecting(world: &mut World) {
        *world.resource_mut::<core_discovery::RoomRoster>() =
            roster_with(core_discovery::OwnRoomStatus::Connecting, 2, 8);
    }

    fn refresh_connected(world: &mut World) {
        *world.resource_mut::<core_discovery::RoomRoster>() =
            roster_with(core_discovery::OwnRoomStatus::Connected, 3, 8);
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(core_discovery::RoomRoster::default());
        app.add_systems(
            Startup,
            (
                spawn_mock_screen,
                ui::roster::bevy_systems::spawn_roster_bubble,
            ),
        );
        app.add_systems(
            Update,
            (
                ui::roster::bevy_systems::refresh_roster_status,
                refresh_roster_tooltips_text,
            ),
        );
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
        ));
    }

    #[test]
    fn refresh_roster_tooltips_text_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_roster_tooltips_text"),
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
