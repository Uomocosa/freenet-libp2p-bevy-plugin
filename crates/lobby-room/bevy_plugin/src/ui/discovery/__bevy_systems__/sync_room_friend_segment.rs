use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    FRIEND_BADGE_COLOR, FRIEND_PULSE_GROW, FRIEND_PULSE_RING_ALPHA, FRIEND_PULSE_RING_BORDER,
    FRIEND_PULSE_RING_INSET, FRIENDS_SLOT, RoomFriendSegment, RoomFriendSegmentContent,
    RoomFriendSegmentFade, RoomPillRing, RoomRowParts,
};

pub fn sync_room_friend_segment(
    mut commands: Commands,
    rows: Query<(Entity, &ui::roll::RollMachine, &RoomRowParts)>,
    mut segments: Query<(Entity, &RoomFriendSegment, &mut Node)>,
    contents: Query<(&RoomFriendSegmentContent, &ComputedNode)>,
    mut fades: Query<(&RoomFriendSegmentFade, &mut BackgroundColor)>,
    mut rings: Query<(&RoomPillRing, &mut UiTransform, &mut BorderColor)>,
) {
    for (pill, machine, parts) in &rows {
        let friends = machine.slot_value(FRIENDS_SLOT);
        let is_revealing = matches!(&machine.phase, ui::roll::Phase::Reveal { .. });
        let progress = machine.reveal_progress();
        let full_width = contents
            .iter()
            .find(|(content, _)| ***content == pill)
            .map_or(0.0, |(_, node)| node.size().x);
        let existing = segments
            .iter()
            .find(|(_, segment, _)| ***segment == pill)
            .map(|(entity, _, _)| entity);
        if friends == 0 {
            if let Some(entity) = existing
                && !is_revealing
            {
                commands.entity(entity).despawn();
            }
            continue;
        }
        if existing.is_none() {
            let mut segment = Entity::PLACEHOLDER;
            commands.entity(parts.bubble).with_children(|bubble| {
                segment = ui::discovery::bevy_systems::spawn_friend_segment(bubble, pill, machine);
            });
            commands.entity(parts.bubble).insert_children(0, &[segment]);
        }
        let Some((_, _, mut node)) = segments
            .iter_mut()
            .find(|(_, segment, _)| ***segment == pill)
        else {
            continue;
        };
        node.width = if is_revealing {
            Val::Px(full_width * ui::roll::bevy_systems::ease_out_cubic(progress))
        } else {
            Val::Auto
        };
    }
    for (pill, machine, parts) in &rows {
        if machine.slot_value(FRIENDS_SLOT) == 0
            || !matches!(&machine.phase, ui::roll::Phase::Reveal { .. })
            || rings.iter().any(|(ring, _, _)| **ring == pill)
        {
            continue;
        }
        commands.entity(parts.bubble).with_children(|bubble| {
            bubble.spawn((
                RoomPillRing(pill),
                UiTransform::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(FRIEND_PULSE_RING_INSET),
                    right: Val::Px(FRIEND_PULSE_RING_INSET),
                    top: Val::Px(FRIEND_PULSE_RING_INSET),
                    bottom: Val::Px(FRIEND_PULSE_RING_INSET),
                    border: UiRect::all(Val::Px(FRIEND_PULSE_RING_BORDER)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BorderColor::all(FRIEND_BADGE_COLOR.with_alpha(0.0)),
            ));
        });
    }
    for (ring, mut transform, mut border) in &mut rings {
        let Some((_, machine, _)) = rows.iter().find(|(pill, _, _)| *pill == **ring) else {
            continue;
        };
        let is_revealing = matches!(&machine.phase, ui::roll::Phase::Reveal { .. });
        let eased = ui::roll::bevy_systems::ease_out_cubic(machine.reveal_progress());
        if is_revealing {
            transform.scale = Vec2::splat((FRIEND_PULSE_GROW - 1.0).mul_add(eased, 1.0));
            *border = BorderColor::all(
                FRIEND_BADGE_COLOR.with_alpha(FRIEND_PULSE_RING_ALPHA * (1.0 - eased)),
            );
        } else {
            transform.scale = Vec2::ONE;
            *border = BorderColor::all(FRIEND_BADGE_COLOR.with_alpha(0.0));
        }
    }
    for (fade, mut background) in &mut fades {
        let Some((_, machine, _)) = rows.iter().find(|(pill, _, _)| *pill == fade.pill) else {
            continue;
        };
        let alpha = if matches!(&machine.phase, ui::roll::Phase::Reveal { .. }) {
            ui::roll::bevy_systems::ease_out_cubic(machine.reveal_progress())
        } else {
            1.0
        };
        *background = BackgroundColor(fade.base.with_alpha(fade.base.alpha() * alpha));
    }
}

#[cfg(test)]
mod tests {
    use super::sync_room_friend_segment;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    const PULSE_PREVIEW_FPS: u32 = 30;
    const PULSE_PREVIEW_FRAMES: u32 = 132;
    const PULSE_SLOW_MOTION: f32 = 0.25;
    const FIRST_FRIEND_DELAY_SECS: f32 = 0.10;
    const SECOND_FRIEND_DELAY_SECS: f32 = 0.37;

    #[test]
    fn sync_room_friend_segment_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("sync_room_friend_segment"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
                    name: core_discovery::RoomName::from("alpha"),
                    occupants: 2,
                    room_size: core_discovery::RoomSize::try_from(8).expect("room size"),
                    occupancy: core_discovery::RoomOccupancy::HasSpace,
                    friends: 1,
                }]));
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    commands.spawn((
                        ChildOf(root),
                        ui::discovery::RoomList,
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                    ));
                });
                app.add_systems(
                    Update,
                    (
                        ui::discovery::bevy_systems::rebuild_room_list,
                        sync_room_friend_segment,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("badge at rest"),
                    apply: |world: &mut World| {
                        let mut machines = world.query::<&mut ui::roll::RollMachine>();
                        for mut machine in machines.iter_mut(world) {
                            **machine = ui::roll::State::new(
                                vec![2, 8, 1],
                                Some(ui::discovery::FRIENDS_SLOT),
                            );
                        }
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("badge revealing"),
                    apply: |world: &mut World| {
                        let mut machines = world.query::<&mut ui::roll::RollMachine>();
                        for mut machine in machines.iter_mut(world) {
                            machine.numbers = vec![2, 8, 0];
                            machine.target = vec![2, 8, 1];
                            machine.phase = ui::roll::Phase::Reveal {
                                slot: ui::discovery::FRIENDS_SLOT,
                                revealed_numbers: vec![2, 8, 1],
                                elapsed_secs: 0.225,
                            };
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

    #[test]
    fn sync_room_friend_segment_pulse_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("sync_room_friend_segment_pulse"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
                    name: core_discovery::RoomName::from("alpha"),
                    occupants: 2,
                    room_size: core_discovery::RoomSize::try_from(8).expect("room size"),
                    occupancy: core_discovery::RoomOccupancy::HasSpace,
                    friends: 0,
                }]));
                app.add_plugins(ui::roll::UiPlugin);
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    commands.spawn((
                        ChildOf(root),
                        ui::discovery::RoomList,
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                    ));
                });
                app.configure_sets(
                    Update,
                    ui::roll::RollSet::Drive.after(ui::discovery::bevy_systems::rebuild_room_list),
                );
                app.add_systems(
                    Update,
                    (
                        (
                            |clock: Res<lele_bevy_preview::scene::Clock>,
                             time: Res<Time>,
                             mut rows: ResMut<core_discovery::LobbyRows>| {
                                let Some(elapsed) = clock.elapsed(&time) else {
                                    return;
                                };
                                let (occupants, friends) = if elapsed < FIRST_FRIEND_DELAY_SECS {
                                    (2, 0)
                                } else if elapsed < SECOND_FRIEND_DELAY_SECS {
                                    (2, 1)
                                } else {
                                    (3, 2)
                                };
                                let core_discovery::LobbyRows(row_list) = &mut *rows;
                                let Some(row) = row_list.first_mut() else {
                                    return;
                                };
                                if row.occupants != occupants || row.friends != friends {
                                    row.occupants = occupants;
                                    row.friends = friends;
                                }
                            },
                        )
                            .before(ui::discovery::bevy_systems::rebuild_room_list),
                        ui::discovery::bevy_systems::rebuild_room_list,
                        sync_room_friend_segment.after(ui::roll::RollSet::Drive),
                    ),
                );
                app.add_systems(
                    PostStartup,
                    lele_bevy_preview::scene::bevy_systems::center_roots,
                );
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("settled: three of eight, two friends"),
                apply: |world: &mut World| {
                    let mut rows = world.resource_mut::<core_discovery::LobbyRows>();
                    if let Some(row) = rows.0.first_mut() {
                        row.occupants = 3;
                        row.friends = 2;
                    }
                    let mut machines = world.query::<&mut ui::roll::RollMachine>();
                    for mut machine in machines.iter_mut(world) {
                        **machine =
                            ui::roll::State::new(vec![3, 8, 2], Some(ui::discovery::FRIENDS_SLOT));
                    }
                },
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("first friend pulse then one roll (0.25x)"),
                frames: PULSE_PREVIEW_FRAMES,
                fps: PULSE_PREVIEW_FPS,
                apply: |world: &mut World| {
                    world
                        .resource_mut::<Time<Virtual>>()
                        .set_relative_speed(PULSE_SLOW_MOTION);
                    lele_bevy_preview::scene::Clock::start(world);
                },
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
