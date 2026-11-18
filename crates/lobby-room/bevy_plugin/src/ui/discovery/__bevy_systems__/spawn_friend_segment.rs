use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    FRIEND_BADGE_COLOR, FRIEND_DIVIDER_COLOR, FRIEND_DIVIDER_HEIGHT, FRIEND_DIVIDER_WIDTH,
    FRIEND_NUMBER_STYLE, FRIENDS_SLOT, RoomFriendSegment, RoomFriendSegmentContent,
    RoomFriendSegmentFade,
};

pub fn spawn_friend_segment(
    pill: &mut ChildSpawnerCommands,
    row: Entity,
    state: &ui::roll::State,
) -> Entity {
    let friends = state.slot_value(FRIENDS_SLOT);
    let clip_width = if matches!(&state.phase, ui::roll::Phase::Reveal { .. }) {
        Val::Px(0.0)
    } else {
        Val::Auto
    };
    let mut segment = pill.spawn((
        RoomFriendSegment(row),
        Node {
            width: clip_width,
            overflow: Overflow::clip_x(),
            flex_shrink: 0.0,
            ..default()
        },
    ));
    let segment_entity = segment.id();
    segment.with_children(|clip| {
        clip.spawn((
            RoomFriendSegmentContent(row),
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .with_children(|content| {
            content
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(1.0),
                    ..default()
                })
                .with_children(|person| {
                    person.spawn((
                        RoomFriendSegmentFade {
                            base: FRIEND_BADGE_COLOR,
                            pill: row,
                        },
                        Node {
                            width: Val::Px(4.0),
                            height: Val::Px(4.0),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(FRIEND_BADGE_COLOR),
                    ));
                    person.spawn((
                        RoomFriendSegmentFade {
                            base: FRIEND_BADGE_COLOR,
                            pill: row,
                        },
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
                content,
                row,
                FRIENDS_SLOT,
                friends,
                FRIEND_NUMBER_STYLE,
            );
            content.spawn((
                RoomFriendSegmentFade {
                    base: FRIEND_DIVIDER_COLOR,
                    pill: row,
                },
                Node {
                    width: Val::Px(FRIEND_DIVIDER_WIDTH),
                    height: Val::Px(FRIEND_DIVIDER_HEIGHT),
                    ..default()
                },
                BackgroundColor(FRIEND_DIVIDER_COLOR),
            ));
        });
    });
    segment_entity
}

#[cfg(test)]
mod tests {
    use super::spawn_friend_segment;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn spawn_friend_segment_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_friend_segment"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_plugins(ui::roll::UiPlugin);
                app.add_systems(Startup, |mut commands: Commands| {
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
                    for (occupancy, friends, color) in [
                        ("3/8", 2_u32, ui::discovery::ROOM_BUBBLE_HAS_SPACE_COLOR),
                        ("4/4", 12_u32, ui::discovery::ROOM_BUBBLE_FULL_COLOR),
                    ] {
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
                                    let state = ui::roll::State::new(
                                        vec![0, 0, friends],
                                        Some(ui::discovery::FRIENDS_SLOT),
                                    );
                                    spawn_friend_segment(bubble, pill, &state);
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
                });
                app.add_systems(
                    PostStartup,
                    lele_bevy_preview::scene::bevy_systems::center_roots,
                );
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("friends inside the occupancy pill"),
                apply: |_world: &mut World| {},
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
