use crate::discovery as core_discovery;
use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::ui::basic;
use ui::discovery::{
    BUTTON_LABEL_COLOR, HOVER_OUTLINE_WIDTH, HoverGlow, ROOM_BUBBLE_EMPTY_COLOR,
    ROOM_BUBBLE_FULL_COLOR, ROOM_BUBBLE_HAS_SPACE_COLOR, ROOM_NAME_WIDTH, ROOM_ROW_WIDTH,
    RoomBubble, RoomButton, RoomNameClip, RoomNameScroller,
};

pub fn spawn_room_row(rooms: &mut ChildSpawnerCommands, row: &core_discovery::LobbyRow) {
    let bubble_color = match row.occupancy {
        core_discovery::RoomOccupancy::HasSpace => ROOM_BUBBLE_HAS_SPACE_COLOR,
        core_discovery::RoomOccupancy::Empty => ROOM_BUBBLE_EMPTY_COLOR,
        core_discovery::RoomOccupancy::Full => ROOM_BUBBLE_FULL_COLOR,
    };
    rooms
        .spawn((
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
        ))
        .with_children(|room| {
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
                        scroller.spawn((
                            Text::new(row.name.as_str().to_string()),
                            TextLayout::no_wrap(),
                        ));
                    });
            });
            room.spawn((Node {
                flex_grow: 1.0,
                ..default()
            },));
            room.spawn((
                RoomBubble,
                Node {
                    padding: UiRect {
                        left: Val::Px(7.0),
                        right: Val::Px(14.0),
                        top: Val::Px(2.5),
                        bottom: Val::Px(2.5),
                    },
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(bubble_color),
            ))
            .with_children(|bubble| {
                bubble.spawn((
                    Text::new(format!("{}/{}", row.occupants, row.room_size)),
                    TextFont {
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(BUTTON_LABEL_COLOR),
                ));
                if row.friends > 0 {
                    ui::discovery::bevy_systems::spawn_friend_badge(bubble, row.friends);
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use super::spawn_room_row;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use freenet_libp2p_bevy_core::identity;

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

    fn sample_row() -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: identity::RoomName::from("alpha"),
            occupants: 3,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 2,
        }
    }

    fn spawn_room_with_row(mut commands: Commands) {
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
            spawn_room_row(&mut rooms.reborrow(), &sample_row());
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
                label: String::from("row with friends"),
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
        app.add_systems(Startup, spawn_room_with_row);
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
    }
}
