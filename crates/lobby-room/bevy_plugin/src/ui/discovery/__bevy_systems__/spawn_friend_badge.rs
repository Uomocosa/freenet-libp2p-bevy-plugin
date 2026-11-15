use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    BUTTON_LABEL_COLOR, FRIEND_BADGE_BG_COLOR, FRIEND_BADGE_COLOR, RoomFriendBadge,
};

pub fn spawn_friend_badge(bubble: &mut ChildSpawnerCommands, friends: usize) {
    bubble
        .spawn((
            RoomFriendBadge,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(-14.0),
                right: Val::Px(-8.0),
                padding: UiRect::all(Val::Px(3.0)),
                column_gap: Val::Px(2.0),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(FRIEND_BADGE_BG_COLOR),
        ))
        .with_children(|badge| {
            badge
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(1.0),
                    ..default()
                },))
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
            badge.spawn((
                Text::new(friends.to_string()),
                TextFont {
                    font_size: FontSize::Px(9.0),
                    ..default()
                },
                TextColor(BUTTON_LABEL_COLOR),
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::spawn_friend_badge;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            commands
                .spawn(ui::discovery::RoomBubble)
                .with_children(|bubble| {
                    spawn_friend_badge(bubble, 2);
                });
        });
        app.update();
        let mut badges = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomFriendBadge>>();
        assert_eq!(badges.iter(app.world()).count(), 1);
    }

    fn spawn_bubble_with_badge(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands
            .spawn((
                ChildOf(root),
                ui::discovery::RoomBubble,
                Node {
                    width: Val::Px(44.0),
                    height: Val::Px(20.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::axes(Val::Px(7.0), Val::Px(2.5)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(ui::discovery::ROOM_BUBBLE_HAS_SPACE_COLOR),
            ))
            .with_children(|bubble| {
                bubble.spawn((
                    Text::new("3/8".to_string()),
                    TextFont {
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(ui::discovery::BUTTON_LABEL_COLOR),
                ));
                spawn_friend_badge(&mut bubble.reborrow(), 2);
            });
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_bubble_with_badge);
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
    }

    #[test]
    fn spawn_friend_badge_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_friend_badge"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("badge on the bubble"),
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
