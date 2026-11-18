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

    #[test]
    fn refresh_roster_tooltips_text_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_roster_tooltips_text"),
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
                            ));
                        },
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
