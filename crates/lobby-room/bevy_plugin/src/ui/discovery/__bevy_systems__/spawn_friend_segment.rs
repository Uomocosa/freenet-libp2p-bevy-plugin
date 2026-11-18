use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    FRIEND_BADGE_COLOR, FRIEND_DIVIDER_COLOR, FRIEND_DIVIDER_HEIGHT, FRIEND_DIVIDER_WIDTH,
    FRIEND_NUMBER_STYLE, FRIENDS_SLOT, RoomFriendSegment,
};

pub fn spawn_friend_segment(pill: &mut ChildSpawnerCommands, row: Entity, friends: u32) {
    pill.spawn((
        RoomFriendSegment,
        Node {
            column_gap: Val::Px(4.0),
            align_items: AlignItems::Center,
            ..default()
        },
    ))
    .with_children(|segment| {
        segment
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(1.0),
                ..default()
            })
            .with_children(|person| {
                person.spawn((
                    Node {
                        width: Val::Px(4.0),
                        height: Val::Px(4.0),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(FRIEND_BADGE_COLOR),
                ));
                person.spawn((
                    Node {
                        width: Val::Px(10.0),
                        height: Val::Px(5.0),
                        border_radius: BorderRadius {
                            top_left: CornerRadius::circular(Val::Px(5.0)),
                            top_right: CornerRadius::circular(Val::Px(5.0)),
                            bottom_left: CornerRadius::ZERO,
                            bottom_right: CornerRadius::ZERO,
                        },
                        ..default()
                    },
                    BackgroundColor(FRIEND_BADGE_COLOR),
                ));
            });
        ui::roll::bevy_systems::spawn_roll_number(
            segment,
            row,
            FRIENDS_SLOT,
            friends,
            FRIEND_NUMBER_STYLE,
        );
        segment.spawn((
            Node {
                width: Val::Px(FRIEND_DIVIDER_WIDTH),
                height: Val::Px(FRIEND_DIVIDER_HEIGHT),
                ..default()
            },
            BackgroundColor(FRIEND_DIVIDER_COLOR),
        ));
    });
}

#[cfg(test)]
mod tests {
    use super::spawn_friend_segment;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            let row = commands.spawn_empty().id();
            commands
                .spawn(ui::discovery::RoomBubble)
                .with_children(|bubble| {
                    spawn_friend_segment(bubble, row, 2);
                });
        });
        app.update();
        let mut segments = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomFriendSegment>>();
        assert_eq!(segments.iter(app.world()).count(), 1);
        let mut numbers = app.world_mut().query::<&ui::roll::RollNumber>();
        let number = numbers.single(app.world()).expect("the friend number");
        assert_eq!(
            (number.slot, number.value),
            (ui::discovery::FRIENDS_SLOT, 2)
        );
    }

    fn spawn_pill(
        commands: &mut Commands,
        list: Entity,
        occupancy: &str,
        friends: u32,
        color: Color,
    ) {
        commands
            .spawn((
                ChildOf(list),
                ui::discovery::RoomBubble,
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.0),
                    padding: UiRect::axes(Val::Px(7.0), Val::Px(2.5)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(color),
            ))
            .with_children(|bubble| {
                let pill = bubble.target_entity();
                if friends > 0 {
                    spawn_friend_segment(bubble, pill, friends);
                }
                bubble.spawn((
                    Text::new(occupancy.to_string()),
                    TextFont {
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(ui::discovery::BUTTON_LABEL_COLOR),
                ));
            });
    }

    fn spawn_pills(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        let list = commands
            .spawn((
                ChildOf(root),
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
            ))
            .id();
        spawn_pill(
            &mut commands,
            list,
            "3/8",
            2,
            ui::discovery::ROOM_BUBBLE_HAS_SPACE_COLOR,
        );
        spawn_pill(
            &mut commands,
            list,
            "4/4",
            12,
            ui::discovery::ROOM_BUBBLE_FULL_COLOR,
        );
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_build(app: &mut App) {
        app.add_plugins(ui::roll::UiPlugin);
        app.add_systems(Startup, spawn_pills);
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
    }

    #[test]
    fn spawn_friend_segment_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_friend_segment"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("friends inside the occupancy pill"),
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
}
