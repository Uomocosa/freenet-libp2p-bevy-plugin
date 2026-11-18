use crate::spectator;
use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;

use ui::roster::{SpectatorPill, SpectatorPillTooltip};

pub fn refresh_spectator_tooltip(
    count: Option<Res<spectator::SpectatorCount>>,
    pill: Single<&Hovered, With<SpectatorPill>>,
    tip: Single<(&mut Visibility, &mut Text), With<SpectatorPillTooltip>>,
) {
    let Some(count) = count else {
        return;
    };
    let watching = **count;
    let (mut tip_visibility, mut tip_text) = tip.into_inner();
    let wanted_watching_label = if watching == 1 {
        String::from("1 player watching")
    } else {
        format!("{watching} players watching")
    };
    if **tip_text != wanted_watching_label {
        **tip_text = wanted_watching_label;
    }
    let pill = pill.into_inner();
    let wanted_visibility = if pill.get() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if *tip_visibility != wanted_visibility {
        *tip_visibility = wanted_visibility;
    }
}

#[cfg(test)]
mod tests {
    use super::refresh_spectator_tooltip;
    use crate::discovery as core_discovery;
    use crate::spectator;
    use crate::ui;
    use bevy::picking::backend::HitData;
    use bevy::picking::hover::PointerCaptureMap;
    use bevy::picking::pointer::PointerId;
    use bevy::prelude::*;

    const SYNTHETIC_HIT: HitData = HitData {
        camera: Entity::PLACEHOLDER,
        depth: 0.0,
        position: None,
        normal: None,
        extra: None,
    };

    #[test]
    fn refresh_spectator_tooltip_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_spectator_tooltip"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_discovery::RoomRoster {
                    own: core_discovery::OwnRoomStatus::Connected,
                    count: 3,
                    max: 8,
                    entries: Vec::new(),
                });
                app.insert_resource(spectator::SpectatorCount(1));
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
                        ui::roster::bevy_systems::refresh_spectator_pill,
                        refresh_spectator_tooltip,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("not hovered"),
                    apply: |world: &mut World| {
                        drop(
                            world
                                .resource_mut::<PointerCaptureMap>()
                                .release(PointerId::Mouse),
                        );
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("hovered"),
                    apply: |world: &mut World| {
                        let pill = world
                            .query_filtered::<Entity, With<ui::roster::SpectatorPill>>()
                            .iter(world)
                            .next();
                        if let Some(pill) = pill {
                            world.resource_mut::<PointerCaptureMap>().capture(
                                PointerId::Mouse,
                                pill,
                                SYNTHETIC_HIT,
                            );
                        }
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
