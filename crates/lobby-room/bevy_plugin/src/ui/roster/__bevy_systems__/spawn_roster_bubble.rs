use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use ui::roster::{
    ROSTER_BUBBLE_LEFT, ROSTER_BUBBLE_ROOM_BAR_CLEARANCE, ROSTER_BUBBLE_Z, ROSTER_COUNT_FONT_SIZE,
    ROSTER_DISCONNECTED_COLOR, ROSTER_DOT_SIZE, ROSTER_LABEL_COLOR, ROSTER_LIST_WIDTH,
    ROSTER_TOOLTIP_FONT_SIZE, ROSTER_TOOLTIP_TEXT_COLOR, RosterBubble, RosterCount,
    RosterCountTooltip, RosterDot, RosterDotTooltip, RosterList, RosterPill, SpectatorPillTooltip,
};

pub fn spawn_roster_bubble(mut commands: Commands) {
    let tooltip = |text: &str| {
        (
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(32.0),
                left: Val::Px(0.0),
                padding: UiRect::all(Val::Px(6.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(ROSTER_TOOLTIP_FONT_SIZE),
                ..default()
            },
            TextColor(ROSTER_TOOLTIP_TEXT_COLOR),
            BackgroundColor(ui::constants::DRAWER_PANEL_COLOR),
        )
    };
    commands
        .spawn((
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
        ))
        .with_children(|bubble| {
            bubble
                .spawn((Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                },))
                .with_children(|row| {
                    row.spawn((
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
                        BackgroundColor(ui::constants::DRAWER_PANEL_COLOR),
                    ))
                    .with_children(|pill| {
                        pill.spawn((
                            RosterDot,
                            Hovered::default(),
                            Node {
                                width: Val::Px(ROSTER_DOT_SIZE),
                                height: Val::Px(ROSTER_DOT_SIZE),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(ROSTER_DISCONNECTED_COLOR),
                        ));
                        pill.spawn((
                            RosterCount,
                            Hovered::default(),
                            Text::new(""),
                            TextFont {
                                font_size: FontSize::Px(ROSTER_COUNT_FONT_SIZE),
                                ..default()
                            },
                            TextColor(ROSTER_LABEL_COLOR),
                        ));
                    });
                    ui::roster::bevy_systems::spawn_spectator_pill(row);
                });
            bubble.spawn((RosterDotTooltip, tooltip("Disconnected")));
            bubble.spawn((RosterCountTooltip, tooltip("Peers in the room: 0/0")));
            bubble.spawn((SpectatorPillTooltip, tooltip("0 players watching")));
            bubble.spawn((
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
                BackgroundColor(ui::constants::DRAWER_PANEL_COLOR),
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::spawn_roster_bubble;
    use crate::discovery as core_discovery;
    use crate::spectator;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn spawn_roster_bubble_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_roster_bubble"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_discovery::RoomRoster {
                    own: core_discovery::OwnRoomStatus::Connected,
                    count: 3,
                    max: 8,
                    entries: Vec::new(),
                });
                app.insert_resource(spectator::SpectatorCount(2));
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
                        },
                        spawn_roster_bubble,
                    ),
                );
                app.add_systems(
                    Update,
                    (
                        ui::roster::bevy_systems::refresh_roster_status,
                        ui::roster::bevy_systems::refresh_spectator_pill,
                        ui::roster::bevy_systems::refresh_spectator_tooltip,
                    ),
                );
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("collapsed pill"),
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
