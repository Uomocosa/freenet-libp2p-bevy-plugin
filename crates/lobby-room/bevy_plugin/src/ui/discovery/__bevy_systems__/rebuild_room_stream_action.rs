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
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::rebuild_room_stream_action;

    fn row(name: &str, occupancy: core_discovery::RoomOccupancy) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants: 0,
            room_size: core_discovery::RoomSize::try_from(4).expect("4"),
            occupancy,
            friends: 0,
        }
    }

    fn app_with(room: &str, rows: Vec<core_discovery::LobbyRow>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ui::discovery::StreamedRoom(Some(
            core_discovery::RoomName::from(room),
        )));
        app.insert_resource(core_discovery::LobbyRows(rows));
        app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_stream);
        app.update();
        app.add_systems(Update, rebuild_room_stream_action);
        app
    }

    fn count<C: Component>(app: &mut App) -> usize {
        let mut query = app.world_mut().query_filtered::<(), With<C>>();
        query.iter(app.world()).count()
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(
            "alpha",
            vec![row("alpha", core_discovery::RoomOccupancy::HasSpace)],
        );
        app.update();
        assert_eq!(count::<ui::discovery::EnterRoomButton>(&mut app), 1);
        assert_eq!(count::<ui::discovery::RoomFullBadge>(&mut app), 0);
    }

    #[test]
    fn a_full_streamed_room_shows_the_full_badge_and_no_join_button() {
        let mut app = app_with(
            "gamma",
            vec![row("gamma", core_discovery::RoomOccupancy::Full)],
        );
        app.update();
        assert_eq!(count::<ui::discovery::RoomFullBadge>(&mut app), 1);
        assert_eq!(count::<ui::discovery::EnterRoomButton>(&mut app), 0);
    }

    #[test]
    fn the_join_button_joins_the_streamed_room() {
        let mut app = app_with(
            "alpha",
            vec![row("alpha", core_discovery::RoomOccupancy::HasSpace)],
        );
        app.add_message::<core_discovery::Command>();
        app.add_systems(
            Update,
            ui::discovery::bevy_systems::handle_join.after(rebuild_room_stream_action),
        );
        app.update();
        let mut buttons = app
            .world_mut()
            .query_filtered::<Entity, With<ui::discovery::EnterRoomButton>>();
        let button = buttons.single(app.world()).expect("join button");
        app.world_mut().entity_mut(button).insert(Pressed);
        app.update();
        let messages = app.world().resource::<Messages<core_discovery::Command>>();
        let mut cursor = messages.get_cursor();
        assert_eq!(
            cursor.read(messages).cloned().collect::<Vec<_>>(),
            vec![core_discovery::Command::Join(
                core_discovery::RoomName::from("alpha")
            )]
        );
    }
}

#[cfg(test)]
mod png_tests {
    use super::rebuild_room_stream_action;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    fn row(name: &str, occupancy: core_discovery::RoomOccupancy) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants: 0,
            room_size: core_discovery::RoomSize::try_from(4).expect("4"),
            occupancy,
            friends: 0,
        }
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(ui::discovery::StreamDisplay::default());
        app.insert_resource(ui::discovery::StreamedRoom(Some(
            core_discovery::RoomName::from("alpha"),
        )));
        app.insert_resource(core_discovery::LobbyRows(vec![
            row("alpha", core_discovery::RoomOccupancy::HasSpace),
            row("gamma", core_discovery::RoomOccupancy::Full),
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
    }

    fn stream_alpha(world: &mut World) {
        **world.resource_mut::<ui::discovery::StreamedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
    }

    fn stream_gamma(world: &mut World) {
        **world.resource_mut::<ui::discovery::StreamedRoom>() =
            Some(core_discovery::RoomName::from("gamma"));
    }

    #[test]
    fn rebuild_room_stream_action_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_room_stream_action"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("join button"),
                    apply: stream_alpha,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("room full"),
                    apply: stream_gamma,
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
