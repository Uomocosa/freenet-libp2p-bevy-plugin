use crate::spectator;
use crate::ui;
use bevy::prelude::*;

use ui::roster::{SpectatorCountText, SpectatorPill};

pub fn refresh_spectator_pill(
    count: Option<Res<spectator::SpectatorCount>>,
    pill: Single<&mut Visibility, With<SpectatorPill>>,
    text: Single<&mut Text, With<SpectatorCountText>>,
) {
    let Some(count) = count else {
        return;
    };
    let watching = **count;
    let wanted_visibility = if watching > 0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let mut pill = pill.into_inner();
    if *pill != wanted_visibility {
        *pill = wanted_visibility;
    }
    let mut text = text.into_inner();
    let wanted_count = watching.to_string();
    if **text != wanted_count {
        **text = wanted_count;
    }
}

#[cfg(test)]
mod tests {
    use super::refresh_spectator_pill;
    use crate::discovery as core_discovery;
    use crate::spectator;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn refresh_spectator_pill_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_spectator_pill"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_discovery::RoomRoster {
                    own: core_discovery::OwnRoomStatus::Connected,
                    count: 3,
                    max: 8,
                    entries: Vec::new(),
                });
                app.insert_resource(spectator::SpectatorCount::default());
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
                        ui::roster::bevy_systems::refresh_roster_status,
                        refresh_spectator_pill,
                        ui::roster::bevy_systems::refresh_spectator_tooltip,
                    ),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("no spectators"),
                    apply: |world: &mut World| {
                        world.insert_resource(spectator::SpectatorCount(0));
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("one spectator"),
                    apply: |world: &mut World| {
                        world.insert_resource(spectator::SpectatorCount(1));
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("twelve spectators"),
                    apply: |world: &mut World| {
                        world.insert_resource(spectator::SpectatorCount(12));
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
