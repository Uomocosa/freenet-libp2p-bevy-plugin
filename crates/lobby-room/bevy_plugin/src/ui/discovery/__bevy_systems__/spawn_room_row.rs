use crate::discovery as core_discovery;
use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use freenet_libp2p_bevy_core::ui as core_ui;

use crate::ui::basic;
use ui::discovery::{
    BUTTON_LABEL_COLOR, FRIENDS_SLOT, HOVER_OUTLINE_WIDTH, HoverGlow, ROOM_BUBBLE_TOOLTIP_BG_COLOR,
    ROOM_BUBBLE_TOOLTIP_FONT_SIZE, ROOM_BUBBLE_TOOLTIP_Z, ROOM_NAME_WIDTH, ROOM_ROW_WIDTH,
    RoomBubble, RoomBubbleTooltip, RoomButton, RoomNameClip, RoomNameScroller, RoomRowParts,
};

pub fn spawn_room_row(rooms: &mut ChildSpawnerCommands, row: &core_discovery::LobbyRow) {
    let bubble_color = ui::discovery::bevy_systems::room_bubble_color(row.occupancy);
    let [occupants, room_size, friends] = ui::discovery::bevy_systems::room_numbers(row);
    let mut row_commands = rooms.spawn((
        RoomButton(row.name.clone()),
        Button,
        Hovered::default(),
        HoverGlow::default(),
        UiTransform::default(),
        Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
        Node {
            width: Val::Px(ROOM_ROW_WIDTH),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
            column_gap: Val::Px(6.0),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(basic::constants::ROOM_COLOR),
    ));
    let row_entity = row_commands.id();
    let roll_state = ui::roll::State::new(vec![occupants, room_size, friends], Some(FRIENDS_SLOT));
    row_commands.insert(ui::roll::RollMachine(roll_state.clone()));
    let (mut name_text, mut bubble_entity) = (Entity::PLACEHOLDER, Entity::PLACEHOLDER);
    let name = (
        Text::new(row.name.as_str().to_string()),
        TextLayout::no_wrap(),
    );
    row_commands.with_children(|room| {
        room.spawn((
            RoomNameClip,
            Node {
                width: Val::Px(ROOM_NAME_WIDTH),
                overflow: Overflow::clip_x(),
                ..default()
            },
        ))
        .with_children(|clip| {
            clip.spawn((RoomNameScroller, UiTransform::default(), Node::default()))
                .with_children(|scroller| {
                    name_text = scroller.spawn(name).id();
                });
        });
        room.spawn(Node {
            flex_grow: 1.0,
            ..default()
        });
        room.spawn((
            RoomBubble,
            Node {
                flex_direction: FlexDirection::Row,
                padding: UiRect::axes(Val::Px(7.0), Val::Px(2.5)),
                column_gap: Val::Px(4.0),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(bubble_color),
        ))
        .with_children(|bubble| {
            bubble_entity = bubble.target_entity();
            if friends > 0 {
                ui::discovery::bevy_systems::spawn_friend_segment(bubble, row_entity, &roll_state);
            }
            ui::discovery::bevy_systems::spawn_occupancy_numbers(
                bubble, row_entity, occupants, room_size,
            );
            bubble.spawn((
                RoomBubbleTooltip,
                core_ui::floating::FloatingPopup {
                    anchor: core_ui::floating::PopupAnchor::Parent,
                    preferred_side: core_ui::floating::PopupSide::Below,
                    cross_align: core_ui::floating::PopupAlign::Centre,
                    gap: core_ui::floating::PopupGap(4.0),
                    edge_margin: core_ui::floating::DEFAULT_POPUP_EDGE_MARGIN,
                },
                Visibility::Hidden,
                GlobalZIndex(ROOM_BUBBLE_TOOLTIP_Z),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(18.0),
                    left: Val::Px(0.0),
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(ROOM_BUBBLE_TOOLTIP_BG_COLOR),
                Text::new(ui::discovery::bevy_systems::occupancy_tooltip_text(row)),
                TextFont::from_font_size(FontSize::Px(ROOM_BUBBLE_TOOLTIP_FONT_SIZE)),
                TextColor(BUTTON_LABEL_COLOR),
                TextLayout::no_wrap(),
            ));
        });
    });
    row_commands.insert(RoomRowParts {
        name_text,
        bubble: bubble_entity,
    });
}

#[cfg(test)]
mod tests {
    use super::spawn_room_row;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            let list = commands.spawn_empty().id();
            commands.entity(list).with_children(|rooms| {
                spawn_room_row(&mut rooms.reborrow(), &sample_row());
            });
        });
        app.update();
        let mut rows = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomButton>>();
        assert_eq!(rows.iter(app.world()).count(), 1);
    }

    #[test]
    fn a_row_carries_one_machine_and_three_rolling_numbers() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            let list = commands.spawn_empty().id();
            commands.entity(list).with_children(|rooms| {
                spawn_room_row(
                    &mut rooms.reborrow(),
                    &row_fixture("alpha", 2, 8, core_discovery::RoomOccupancy::HasSpace, 1),
                );
            });
        });
        app.update();
        let mut machines = app.world_mut().query::<&ui::roll::RollMachine>();
        let machine = machines.single(app.world()).expect("one machine");
        assert_eq!(machine.numbers, vec![2, 8, 1]);
        assert_eq!(machine.reveal_slot, Some(ui::discovery::FRIENDS_SLOT));
        let mut numbers = app.world_mut().query::<&ui::roll::RollNumber>();
        let mut found: Vec<(usize, u32)> = numbers
            .iter(app.world())
            .map(|number| (number.slot, number.value))
            .collect();
        found.sort_unstable();
        assert_eq!(found, vec![(0, 2), (1, 8), (2, 1)]);
    }

    #[test]
    fn a_row_spawned_with_friends_shows_the_segment_without_the_sync_system() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            let list = commands.spawn_empty().id();
            commands.entity(list).with_children(|rooms| {
                spawn_room_row(&mut rooms.reborrow(), &sample_row());
            });
        });
        app.update();
        let mut segments = app
            .world_mut()
            .query::<(&Node, &ui::discovery::RoomFriendSegment)>();
        let (node, _) = segments
            .single(app.world())
            .expect("the friends segment spawns with its row");
        assert_eq!(node.width, Val::Auto, "a fresh segment is not clipped");
        let mut fades = app
            .world_mut()
            .query::<(&ui::discovery::RoomFriendSegmentFade, &BackgroundColor)>();
        let mut fade_count = 0;
        for (fade, background) in fades.iter(app.world()) {
            assert_eq!(background.0, fade.base, "a fresh segment is fully opaque");
            fade_count += 1;
        }
        assert_eq!(fade_count, 3, "dot, badge body and divider");
    }

    fn row_fixture(
        name: &str,
        occupants: usize,
        room_size: u16,
        occupancy: core_discovery::RoomOccupancy,
        friends: usize,
    ) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants,
            room_size: core_discovery::RoomSize::try_from(room_size).expect("room size"),
            occupancy,
            friends,
        }
    }

    fn sample_row() -> core_discovery::LobbyRow {
        row_fixture("alpha", 3, 8, core_discovery::RoomOccupancy::HasSpace, 2)
    }

    fn preview_rows() -> Vec<core_discovery::LobbyRow> {
        vec![
            row_fixture("alpha", 2, 8, core_discovery::RoomOccupancy::HasSpace, 1),
            row_fixture("beta", 0, 4, core_discovery::RoomOccupancy::Empty, 0),
            row_fixture("gamma", 4, 4, core_discovery::RoomOccupancy::Full, 2),
            row_fixture(
                "a very long room name",
                1,
                8,
                core_discovery::RoomOccupancy::HasSpace,
                1,
            ),
        ]
    }

    fn spawn_room_rows(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        let list = commands
            .spawn((
                ChildOf(root),
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
            ))
            .id();
        commands.entity(list).with_children(|rooms| {
            for row in preview_rows() {
                spawn_room_row(&mut rooms.reborrow(), &row);
            }
        });
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn spawn_room_row_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_room_row"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("friends · no friends · full room"),
                apply: preview_noop,
            }],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }

    fn preview_build(app: &mut App) {
        app.add_plugins(ui::roll::UiPlugin);
        app.add_systems(Startup, spawn_room_rows);
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
    }
}
