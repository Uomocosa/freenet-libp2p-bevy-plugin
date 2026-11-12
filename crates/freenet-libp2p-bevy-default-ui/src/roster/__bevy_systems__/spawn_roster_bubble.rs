use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::basic::constants::DRAWER_PANEL_COLOR;
use crate::roster;
use roster::{
    ROSTER_BUBBLE_LEFT, ROSTER_BUBBLE_ROOM_BAR_CLEARANCE, ROSTER_BUBBLE_Z, ROSTER_COUNT_FONT_SIZE,
    ROSTER_DISCONNECTED_COLOR, ROSTER_DOT_SIZE, ROSTER_LABEL_COLOR, ROSTER_LIST_WIDTH,
    ROSTER_TOOLTIP_FONT_SIZE, ROSTER_TOOLTIP_TEXT_COLOR, RosterBubble, RosterCount,
    RosterCountTooltip, RosterDot, RosterDotTooltip, RosterList, RosterPill,
};

pub fn spawn_roster_bubble(mut commands: Commands) {
    let tooltip_node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(32.0),
        left: Val::Px(0.0),
        padding: UiRect::all(Val::Px(6.0)),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        ..default()
    };
    let tooltip_font = TextFont {
        font_size: FontSize::Px(ROSTER_TOOLTIP_FONT_SIZE),
        ..default()
    };
    let tooltip_text_color = TextColor(ROSTER_TOOLTIP_TEXT_COLOR);
    let tooltip_background = BackgroundColor(DRAWER_PANEL_COLOR);
    commands.spawn((
        RosterBubble,
        GlobalZIndex(ROSTER_BUBBLE_Z),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(ROSTER_BUBBLE_ROOM_BAR_CLEARANCE),
            left: Val::Px(ROSTER_BUBBLE_LEFT),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(6.0),
            ..default()
        },
        children![
            (
                RosterPill,
                Button,
                Hovered::default(),
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(7.0),
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(DRAWER_PANEL_COLOR),
                children![
                    (
                        RosterDot,
                        Hovered::default(),
                        Node {
                            width: Val::Px(ROSTER_DOT_SIZE),
                            height: Val::Px(ROSTER_DOT_SIZE),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(ROSTER_DISCONNECTED_COLOR),
                    ),
                    (
                        RosterCount,
                        Hovered::default(),
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(ROSTER_COUNT_FONT_SIZE),
                            ..default()
                        },
                        TextColor(ROSTER_LABEL_COLOR),
                    ),
                ],
            ),
            (
                RosterDotTooltip,
                Visibility::Hidden,
                tooltip_node.clone(),
                Text::new("Disconnected"),
                tooltip_font.clone(),
                tooltip_text_color,
                tooltip_background,
            ),
            (
                RosterCountTooltip,
                Visibility::Hidden,
                tooltip_node,
                Text::new("Peers in the room: 0/0"),
                tooltip_font,
                TextColor(ROSTER_TOOLTIP_TEXT_COLOR),
                BackgroundColor(DRAWER_PANEL_COLOR),
            ),
            (
                RosterList,
                Visibility::Hidden,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    min_width: Val::Px(ROSTER_LIST_WIDTH),
                    ..default()
                },
                BackgroundColor(DRAWER_PANEL_COLOR),
            ),
        ],
    ));
}

#[cfg(test)]
mod tests {
    use super::spawn_roster_bubble;
    use crate::roster;

    use crate::basic::constants::ROOM_COLOR;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, spawn_roster_bubble);
        app.update();
        let mut bubbles = app
            .world_mut()
            .query_filtered::<(), With<roster::RosterBubble>>();
        assert_eq!(bubbles.iter(app.world()).count(), 1);
        let mut dots = app
            .world_mut()
            .query_filtered::<(), With<roster::RosterDot>>();
        assert_eq!(dots.iter(app.world()).count(), 1);
        let mut dot_tips = app
            .world_mut()
            .query_filtered::<(), With<roster::RosterDotTooltip>>();
        assert_eq!(dot_tips.iter(app.world()).count(), 1);
        let mut count_tips = app
            .world_mut()
            .query_filtered::<(), With<roster::RosterCountTooltip>>();
        assert_eq!(count_tips.iter(app.world()).count(), 1);
        let mut lists = app
            .world_mut()
            .query_filtered::<(), With<roster::RosterList>>();
        assert_eq!(lists.iter(app.world()).count(), 1);
    }

    fn spawn_mock_screen(mut commands: Commands) {
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
            BackgroundColor(ROOM_COLOR),
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
    }

    fn preview_roster() -> freenet_libp2p_bevy_core::discovery::RoomRoster {
        freenet_libp2p_bevy_core::discovery::RoomRoster {
            own: freenet_libp2p_bevy_core::discovery::OwnRoomStatus::Connected,
            count: 3,
            max: 8,
            entries: Vec::new(),
        }
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_roster());
        app.add_systems(Startup, (spawn_mock_screen, super::spawn_roster_bubble));
        app.add_systems(Update, roster::bevy_systems::refresh_roster_status);
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn spawn_roster_bubble_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_roster_bubble"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("collapsed pill"),
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
