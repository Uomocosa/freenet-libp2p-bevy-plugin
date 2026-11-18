use crate::discovery as core_discovery;
use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Button;

use ui::discovery::{
    BUTTON_LABEL_COLOR, ENTER_BUTTON_COLOR, EnterRoomButton, HOVER_OUTLINE_WIDTH, HoverGlow,
    PREVIEW_BUTTON_FONT_SIZE, PREVIEW_BUTTON_PADDING_H, PREVIEW_BUTTON_PADDING_V,
    ROOM_FULL_BUTTON_COLOR, ROOM_FULL_BUTTON_LABEL_COLOR, RoomFullBadge, RoomStreamActionSlot,
    StreamedRoom,
};

pub fn rebuild_room_stream_action(
    streamed: Res<StreamedRoom>,
    rows: Option<Res<core_discovery::LobbyRows>>,
    mut commands: Commands,
    action_slot: Option<Single<Entity, With<RoomStreamActionSlot>>>,
    mut shown: Local<Option<(core_discovery::RoomName, bool)>>,
) {
    let Some(action_slot) = action_slot else {
        return;
    };
    let streamed = streamed.into_inner();
    let Some(name) = (**streamed).clone() else {
        return;
    };
    let rows = rows.map(Res::into_inner);
    let Some(row) = rows.and_then(|rows| rows.iter().find(|row| row.name == name).cloned()) else {
        return;
    };
    let is_full = row.occupancy == core_discovery::RoomOccupancy::Full;
    let wanted = (name.clone(), is_full);
    if shown.as_ref() == Some(&wanted) {
        return;
    }
    *shown = Some(wanted);
    commands
        .entity(action_slot.into_inner())
        .despawn_related::<Children>()
        .with_children(|slot| {
            if is_full {
                slot.spawn((
                    RoomFullBadge,
                    InteractionDisabled,
                    Node {
                        padding: UiRect::axes(
                            Val::Px(PREVIEW_BUTTON_PADDING_H),
                            Val::Px(PREVIEW_BUTTON_PADDING_V),
                        ),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(ROOM_FULL_BUTTON_COLOR),
                ))
                .with_children(|badge| {
                    badge.spawn((
                        Text::new("Room full"),
                        TextFont {
                            font_size: FontSize::Px(PREVIEW_BUTTON_FONT_SIZE),
                            ..default()
                        },
                        TextColor(ROOM_FULL_BUTTON_LABEL_COLOR),
                    ));
                });
            } else {
                slot.spawn((
                    EnterRoomButton(name),
                    Button,
                    Hovered::default(),
                    HoverGlow::default(),
                    UiTransform::default(),
                    Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
                    Node {
                        padding: UiRect::axes(
                            Val::Px(PREVIEW_BUTTON_PADDING_H),
                            Val::Px(PREVIEW_BUTTON_PADDING_V),
                        ),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(ENTER_BUTTON_COLOR),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Join"),
                        TextFont {
                            font_size: FontSize::Px(PREVIEW_BUTTON_FONT_SIZE),
                            ..default()
                        },
                        TextColor(BUTTON_LABEL_COLOR),
                    ));
                });
            }
        });
}

#[cfg(test)]
mod png_tests {
    use super::rebuild_room_stream_action;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn rebuild_room_stream_action_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_room_stream_action"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: |app: &mut App| {
                app.insert_resource(ui::discovery::StreamDisplay::default());
                app.insert_resource(ui::discovery::StreamedRoom(Some(
                    core_discovery::RoomName::from("alpha"),
                )));
                app.insert_resource(core_discovery::LobbyRows(vec![
                    core_discovery::LobbyRow {
                        name: core_discovery::RoomName::from("alpha"),
                        occupants: 0,
                        room_size: core_discovery::RoomSize::try_from(4).expect("4"),
                        occupancy: core_discovery::RoomOccupancy::HasSpace,
                        friends: 0,
                    },
                    core_discovery::LobbyRow {
                        name: core_discovery::RoomName::from("gamma"),
                        occupants: 0,
                        room_size: core_discovery::RoomSize::try_from(4).expect("4"),
                        occupancy: core_discovery::RoomOccupancy::Full,
                        friends: 0,
                    },
                ]));
                app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_stream);
                app.add_systems(
                    Update,
                    (
                        ui::discovery::bevy_systems::refresh_room_stream,
                        ui::discovery::bevy_systems::layout_stream_panel,
                        rebuild_room_stream_action,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("join button"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<ui::discovery::StreamedRoom>() =
                            Some(core_discovery::RoomName::from("alpha"));
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("room full"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<ui::discovery::StreamedRoom>() =
                            Some(core_discovery::RoomName::from("gamma"));
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
}
