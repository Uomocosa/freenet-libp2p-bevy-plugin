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
                segment = ui::discovery::bevy_systems::spawn_friend_segment(bubble, pill, friends);
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
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::sync_room_friend_segment;
    use crate::discovery as core_discovery;
    use crate::ui;

    fn lobby_row(occupants: usize, friends: usize) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants,
            room_size: core_discovery::RoomSize::try_from(8).expect("room size"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends,
        }
    }

    fn app_with_rows(rows: Vec<core_discovery::LobbyRow>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )));
        app.insert_resource(core_discovery::LobbyRows(rows));
        app.add_plugins(ui::roll::UiPlugin);
        app.world_mut().spawn(ui::discovery::RoomList);
        app.add_systems(Update, ui::discovery::bevy_systems::rebuild_room_list);
        app.configure_sets(
            Update,
            ui::roll::RollSet::Drive.after(ui::discovery::bevy_systems::rebuild_room_list),
        );
        app.add_systems(
            Update,
            sync_room_friend_segment.after(ui::roll::RollSet::Drive),
        );
        app
    }

    fn set_rows(app: &mut App, rows: Vec<core_discovery::LobbyRow>) {
        *app.world_mut().resource_mut::<core_discovery::LobbyRows>() =
            core_discovery::LobbyRows(rows);
    }

    fn machine_state(app: &mut App) -> ui::roll::State {
        let mut machines = app.world_mut().query::<&ui::roll::RollMachine>();
        machines
            .iter(app.world())
            .next()
            .map(|machine| ui::roll::State::clone(machine))
            .expect("one machine")
    }

    fn segment_count(app: &mut App) -> usize {
        let mut segments = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomFriendSegment>>();
        segments.iter(app.world()).count()
    }

    fn ring_count(app: &mut App) -> usize {
        let mut rings = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomPillRing>>();
        rings.iter(app.world()).count()
    }

    fn first_bubble_child(app: &mut App) -> Entity {
        let mut bubbles = app
            .world_mut()
            .query_filtered::<(Entity, &Children), With<ui::discovery::RoomBubble>>();
        let (_, children) = bubbles.single(app.world()).expect("one bubble");
        *children.first().expect("a first child")
    }

    fn spawn_preview_list(mut commands: Commands) {
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
    }

    fn preview_at_rest(world: &mut World) {
        let mut machines = world.query::<&mut ui::roll::RollMachine>();
        for mut machine in machines.iter_mut(world) {
            **machine = ui::roll::State::new(vec![2, 8, 1], Some(ui::discovery::FRIENDS_SLOT));
        }
    }

    fn preview_revealing(world: &mut World) {
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
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(core_discovery::LobbyRows(vec![lobby_row(2, 1)]));
        app.add_systems(Startup, spawn_preview_list);
        app.add_systems(
            Update,
            (
                ui::discovery::bevy_systems::rebuild_room_list,
                sync_room_friend_segment,
            )
                .chain(),
        );
    }

    #[test]
    fn sync_room_friend_segment_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("sync_room_friend_segment"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("badge at rest"),
                    apply: preview_at_rest,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("badge revealing"),
                    apply: preview_revealing,
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
    fn test_usage() {
        let mut app = app_with_rows(vec![lobby_row(2, 1)]);
        app.update();
        assert_eq!(segment_count(&mut app), 1);
    }

    #[test]
    fn the_revealed_segment_sits_before_the_occupancy_numbers() {
        let mut app = app_with_rows(vec![lobby_row(2, 0)]);
        app.update();
        set_rows(&mut app, vec![lobby_row(2, 1)]);
        app.update();
        app.update();
        let first = first_bubble_child(&mut app);
        assert!(
            app.world()
                .get::<ui::discovery::RoomFriendSegment>(first)
                .is_some(),
            "the friends segment is the bubble's first child"
        );
    }

    #[test]
    fn a_room_with_friends_already_at_spawn_does_not_pulse() {
        let mut app = app_with_rows(vec![lobby_row(2, 2)]);
        app.update();
        for _ in 0..4 {
            app.update();
        }
        let state = machine_state(&mut app);
        assert_eq!(state.phase, ui::roll::Phase::Idle);
        assert_eq!(state.numbers, vec![2, 8, 2]);
        assert_eq!(segment_count(&mut app), 1);
        assert_eq!(ring_count(&mut app), 0, "a spawn-time badge never pulses");
    }

    #[test]
    fn friends_dropping_to_zero_hides_the_segment_after_the_roll() {
        let mut app = app_with_rows(vec![lobby_row(2, 1)]);
        app.update();
        assert_eq!(segment_count(&mut app), 1);
        set_rows(&mut app, vec![lobby_row(2, 0)]);
        app.update();
        assert!(matches!(
            machine_state(&mut app).phase,
            ui::roll::Phase::Roll { .. }
        ));
        assert_eq!(segment_count(&mut app), 1, "the badge rolls down first");
        for _ in 0..6 {
            app.update();
        }
        let state = machine_state(&mut app);
        assert_eq!(state.phase, ui::roll::Phase::Idle);
        assert_eq!(state.numbers, vec![2, 8, 0]);
        assert_eq!(segment_count(&mut app), 0, "the badge hides once idle");
    }

    #[test]
    fn one_roll_after_the_pulse_carries_the_occupant_and_the_second_friend() {
        let mut app = app_with_rows(vec![lobby_row(2, 0)]);
        app.update();
        set_rows(&mut app, vec![lobby_row(2, 1)]);
        app.update();
        assert!(matches!(
            machine_state(&mut app).phase,
            ui::roll::Phase::Reveal { .. }
        ));
        for _ in 0..3 {
            app.update();
        }
        set_rows(&mut app, vec![lobby_row(3, 2)]);
        app.update();
        let state = machine_state(&mut app);
        assert!(
            matches!(state.phase, ui::roll::Phase::Reveal { .. }),
            "the pulse is still running"
        );
        assert_eq!(state.target, vec![3, 8, 2]);
        assert_eq!(state.numbers, vec![2, 8, 0]);
        for _ in 0..4 {
            app.update();
        }
        let state = machine_state(&mut app);
        assert!(
            matches!(
                &state.phase,
                ui::roll::Phase::Roll { from, to, .. }
                    if from == &vec![2, 8, 1] && to == &vec![3, 8, 2]
            ),
            "one roll carries both changes: {:?}",
            state.phase
        );
        for _ in 0..6 {
            app.update();
        }
        let state = machine_state(&mut app);
        assert_eq!(state.phase, ui::roll::Phase::Idle);
        assert_eq!(state.numbers, vec![3, 8, 2]);
        let mut numbers = app.world_mut().query::<&ui::roll::RollNumber>();
        let mut values: Vec<(usize, u32)> = numbers
            .iter(app.world())
            .map(|number| (number.slot, number.value))
            .collect();
        values.sort_unstable();
        assert_eq!(values, vec![(0, 3), (1, 8), (2, 2)]);
    }
}
