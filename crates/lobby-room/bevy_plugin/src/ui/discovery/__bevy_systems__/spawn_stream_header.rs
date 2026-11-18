use crate::ui;
use bevy::prelude::*;
use ui::discovery::{
    BUTTON_COLOR, BUTTON_LABEL_COLOR, RoomStreamActionSlot, RoomStreamCloseButton,
    RoomStreamFullscreenButton, RoomStreamName, RoomStreamOccupancyBubble, RoomStreamOccupancyText,
    STREAM_CONTROL_BAR_COLOR, STREAM_HEADER_GAP, STREAM_HEADER_INSET, STREAM_HEADER_PADDING_H,
    STREAM_HEADER_PADDING_V, STREAM_HEADER_RADIUS, STREAM_OCCUPANCY_FONT_SIZE,
    STREAM_OCCUPANCY_PADDING_H, STREAM_OCCUPANCY_PADDING_V, STREAM_TITLE_FONT_SIZE,
};

pub fn spawn_stream_header(panel: &mut ChildSpawnerCommands) {
    panel
        .spawn((
            ui::discovery::RoomStreamHeader,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(STREAM_HEADER_INSET),
                left: Val::Px(STREAM_HEADER_INSET),
                right: Val::Px(STREAM_HEADER_INSET),
                align_items: AlignItems::Center,
                column_gap: Val::Px(STREAM_HEADER_GAP),
                padding: UiRect::axes(
                    Val::Px(STREAM_HEADER_PADDING_H),
                    Val::Px(STREAM_HEADER_PADDING_V),
                ),
                border_radius: BorderRadius::all(Val::Px(STREAM_HEADER_RADIUS)),
                ..default()
            },
            BackgroundColor(STREAM_CONTROL_BAR_COLOR),
        ))
        .with_children(|header| {
            header.spawn((
                RoomStreamName,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(STREAM_TITLE_FONT_SIZE),
                    ..default()
                },
                TextColor(BUTTON_LABEL_COLOR),
            ));
            header
                .spawn((
                    RoomStreamOccupancyBubble,
                    Node {
                        padding: UiRect::axes(
                            Val::Px(STREAM_OCCUPANCY_PADDING_H),
                            Val::Px(STREAM_OCCUPANCY_PADDING_V),
                        ),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(BUTTON_COLOR),
                ))
                .with_children(|bubble| {
                    bubble.spawn((
                        RoomStreamOccupancyText,
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(STREAM_OCCUPANCY_FONT_SIZE),
                            ..default()
                        },
                        TextColor(BUTTON_LABEL_COLOR),
                    ));
                });
            header.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            header.spawn((RoomStreamActionSlot, Node::default()));
            ui::discovery::bevy_systems::spawn_stream_control_button(
                header,
                RoomStreamFullscreenButton,
                ui::discovery::bevy_systems::spawn_fullscreen_corners,
            );
            ui::discovery::bevy_systems::spawn_stream_control_button(
                header,
                RoomStreamCloseButton,
                ui::discovery::bevy_systems::spawn_close_bars,
            );
        });
}

#[cfg(test)]
mod tests {
    use super::spawn_stream_header;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn spawn_stream_header_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_stream_header"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    commands
                        .spawn((
                            ChildOf(root),
                            Node {
                                width: Val::Px(480.0),
                                height: Val::Px(52.0),
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(ui::discovery::PANEL_COLOR),
                        ))
                        .with_children(|header| spawn_stream_header(&mut header.reborrow()));
                });
                app.add_systems(
                    PostStartup,
                    lele_bevy_preview::scene::bevy_systems::center_roots,
                );
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("stream header"),
                apply: |world: &mut World| {
                    let mut names = world
                        .query_filtered::<&mut Text, With<ui::discovery::RoomStreamName>>();
                    if let Ok(mut name) = names.single_mut(world) {
                        *name = Text::new("alpha");
                    }
                    let mut occupancies = world.query_filtered::<&mut Text, With<ui::discovery::RoomStreamOccupancyText>>();
                    if let Ok(mut occupancy) = occupancies.single_mut(world) {
                        *occupancy = Text::new("2/8");
                    }
                    let mut bubbles = world.query_filtered::<&mut BackgroundColor, With<ui::discovery::RoomStreamOccupancyBubble>>();
                    if let Ok(mut bubble) = bubbles.single_mut(world) {
                        *bubble = BackgroundColor(ui::discovery::ROOM_BUBBLE_HAS_SPACE_COLOR);
                    }
                },
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
