use crate::discovery as core_discovery;
use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Button;

use ui::discovery::{
    BUTTON_LABEL_COLOR, ENTER_BUTTON_COLOR, EnterRoomButton, HOVER_OUTLINE_WIDTH, HoverGlow,
    PREVIEW_BUTTON_FONT_SIZE, PREVIEW_BUTTON_PADDING_H, PREVIEW_BUTTON_PADDING_V,
    ROOM_FULL_BUTTON_COLOR, ROOM_FULL_BUTTON_LABEL_COLOR, RoomFullBadge, RoomPreviewActionSlot,
    SelectedRoom,
};

pub fn rebuild_room_preview_action(
    selection: Res<SelectedRoom>,
    rows: Option<Res<core_discovery::LobbyRows>>,
    mut commands: Commands,
    action_slot: Option<Single<Entity, With<RoomPreviewActionSlot>>>,
    mut shown: Local<Option<bool>>,
) {
    let Some(action_slot) = action_slot else {
        return;
    };
    let selection = selection.into_inner();
    let Some(name) = (**selection).clone() else {
        return;
    };
    let rows = rows.map(Res::into_inner);
    let Some(row) = rows.and_then(|rows| rows.iter().find(|row| row.name == name).cloned()) else {
        return;
    };
    let is_full = row.occupancy == core_discovery::RoomOccupancy::Full;
    if *shown == Some(is_full) {
        return;
    }
    *shown = Some(is_full);
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
                        Text::new("Room is Full"),
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
                        Text::new("Enter"),
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

    use super::rebuild_room_preview_action;

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
        app.insert_resource(ui::discovery::SelectedRoom(Some(
            core_discovery::RoomName::from(room),
        )));
        app.insert_resource(core_discovery::LobbyRows(rows));
        app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_preview);
        app.update();
        app.add_systems(Update, rebuild_room_preview_action);
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
    fn a_full_room_shows_the_full_badge_and_no_enter_button() {
        let mut app = app_with(
            "gamma",
            vec![row("gamma", core_discovery::RoomOccupancy::Full)],
        );
        app.update();
        assert_eq!(count::<ui::discovery::RoomFullBadge>(&mut app), 1);
        assert_eq!(count::<ui::discovery::EnterRoomButton>(&mut app), 0);
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(core_discovery::LobbyRows(vec![
            row("alpha", core_discovery::RoomOccupancy::HasSpace),
            row("gamma", core_discovery::RoomOccupancy::Full),
        ]));
        app.insert_resource(ui::discovery::SelectedRoom(Some(
            core_discovery::RoomName::from("alpha"),
        )));
        app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_preview);
        app.add_systems(
            Update,
            (
                ui::discovery::bevy_systems::refresh_room_preview,
                ui::discovery::bevy_systems::rebuild_room_preview_action,
            ),
        );
    }

    fn select_room(world: &mut World, room: &str) {
        **world.resource_mut::<ui::discovery::SelectedRoom>() =
            Some(core_discovery::RoomName::from(room));
    }

    fn select_alpha(world: &mut World) {
        select_room(world, "alpha");
    }

    fn select_gamma(world: &mut World) {
        select_room(world, "gamma");
    }

    #[test]
    fn rebuild_room_preview_action_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_room_preview_action"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("enter button"),
                    apply: select_alpha,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("room is full"),
                    apply: select_gamma,
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
