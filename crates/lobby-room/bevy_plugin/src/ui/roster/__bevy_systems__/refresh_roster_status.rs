use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::roster::{
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
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn refresh_roster_status_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_roster_status"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_discovery::RoomRoster::default());
                app.add_systems(
                    Startup,
                    (
                        |mut commands: Commands| {
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
                                    children![Text::new("leave")],
                                )],
                            ));
                        },
                        ui::roster::bevy_systems::spawn_roster_bubble,
                    ),
                );
                app.add_systems(
                    Update,
                    (
                        refresh_roster_status,
                        ui::roster::bevy_systems::refresh_roster_tooltips_text,
                    ),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("connecting"),
                    apply: |world: &mut World| {
                        *world.resource_mut::<core_discovery::RoomRoster>() =
                            core_discovery::RoomRoster {
                                own: core_discovery::OwnRoomStatus::Connecting,
                                count: 2,
                                max: 8,
                                entries: Vec::new(),
                            };
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("connected"),
                    apply: |world: &mut World| {
                        *world.resource_mut::<core_discovery::RoomRoster>() =
                            core_discovery::RoomRoster {
                                own: core_discovery::OwnRoomStatus::Connected,
                                count: 3,
                                max: 8,
                                entries: Vec::new(),
                            };
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("swarm lost"),
                    apply: |world: &mut World| {
                        *world.resource_mut::<core_discovery::RoomRoster>() =
                            core_discovery::RoomRoster {
                                own: core_discovery::OwnRoomStatus::Disconnected,
                                count: 3,
                                max: 8,
                                entries: Vec::new(),
                            };
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
