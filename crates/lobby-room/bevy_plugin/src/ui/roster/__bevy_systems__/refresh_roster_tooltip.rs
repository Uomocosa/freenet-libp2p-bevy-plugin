use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;

use ui::roster::{RosterCount, RosterCountTooltip, RosterDot, RosterDotTooltip};

pub fn refresh_roster_tooltip(
    dots: Query<&Hovered, With<RosterDot>>,
    counts: Query<&Hovered, With<RosterCount>>,
    dot_tip: Single<&mut Visibility, (With<RosterDotTooltip>, Without<RosterCountTooltip>)>,
    count_tip: Single<&mut Visibility, (With<RosterCountTooltip>, Without<RosterDotTooltip>)>,
) {
    let is_dot_hovered = dots.iter().any(Hovered::get);
    let is_count_hovered = counts.iter().any(Hovered::get);
    let wanted_dot_tip = if is_dot_hovered {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let mut dot_tip = dot_tip.into_inner();
    if *dot_tip != wanted_dot_tip {
        *dot_tip = wanted_dot_tip;
    }
    let wanted_count_tip = if is_count_hovered {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let mut count_tip = count_tip.into_inner();
    if *count_tip != wanted_count_tip {
        *count_tip = wanted_count_tip;
    }
}

#[cfg(test)]
mod tests {
    use super::refresh_roster_tooltip;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn refresh_roster_tooltip_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_roster_tooltip"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_discovery::RoomRoster {
                    own: core_discovery::OwnRoomStatus::Connected,
                    count: 3,
                    max: 8,
                    entries: Vec::new(),
                });
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
                        ui::roster::bevy_systems::refresh_roster_tooltips_text,
                        refresh_roster_tooltip,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("dot hovered"),
                    apply: lele_bevy_preview::scene::Scene::hover::<ui::roster::RosterDot>,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("count hovered"),
                    apply: lele_bevy_preview::scene::Scene::hover::<ui::roster::RosterCount>,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("not hovered"),
                    apply: |_world: &mut World| {},
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
