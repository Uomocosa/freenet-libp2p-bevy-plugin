//! Replacement lobby UI used by the discovery e2e test (`--ui custom`).
//!
//! It shows that the plugin's default UI can be swapped out entirely: this
//! plugin only reads `DiscoveryState` and writes `discovery::Command`.
#![allow(clippy::needless_pass_by_value)]
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::{Button, TextInput};
use derive_more::Deref;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::discovery;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::ui::discovery::{
    BUTTON_COLOR, BUTTON_LABEL_COLOR, CARET_COLOR, DISABLED_BUTTON_COLOR,
    DISABLED_BUTTON_LABEL_COLOR,
};

const FIELD_COLOR: Color = Color::srgb(0.10, 0.11, 0.14);
const ROOM_COLOR: Color = Color::srgb(0.16, 0.18, 0.22);
const MEMBER_CONNECTED_COLOR: Color = Color::srgb(0.30, 0.82, 0.45);
const MEMBER_CONNECTING_COLOR: Color = Color::srgb(0.87, 0.71, 0.22);
const MEMBER_YOU_COLOR: Color = Color::srgb(0.55, 0.57, 0.64);

pub struct LobbyUi;

impl Plugin for LobbyUi {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn).add_systems(
            Update,
            (
                rebuild_list,
                rebuild_members,
                gate_create,
                on_create,
                on_join,
            ),
        );
    }
}

#[derive(Component)]
pub struct NameField;

#[derive(Component)]
pub struct NewRoomButton;

#[derive(Component)]
struct RoomColumn;

#[derive(Component)]
struct MemberList;

#[derive(Component, Deref)]
pub struct JoinButton(pub identity::RoomName);

type NewRoomButtonParts<'a> = (
    Entity,
    Option<&'a Hovered>,
    Has<InteractionDisabled>,
    &'a mut BackgroundColor,
    &'a Children,
);

fn spawn(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        },
        children![
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                children![
                    (
                        NameField,
                        EditableText::new(""),
                        TextInput,
                        TextCursorStyle {
                            color: CARET_COLOR,
                            ..default()
                        },
                        Node {
                            width: Val::Px(220.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(FIELD_COLOR),
                    ),
                    (
                        NewRoomButton,
                        Button,
                        InteractionDisabled,
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(BUTTON_COLOR),
                        children![Text::new("Create New Room")],
                    ),
                ],
            ),
            (
                RoomColumn,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
            ),
        ],
    ));
    commands.spawn((
        MemberList,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            right: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(FIELD_COLOR),
    ));
}

fn rebuild_list(
    mut commands: Commands,
    state: Option<Res<DiscoveryState>>,
    column: Single<Entity, With<RoomColumn>>,
) {
    if let Some(state) = state.as_ref()
        && !state.is_changed()
    {
        return;
    }
    tracing::info!(
        "lobby ui rooms={}",
        state.as_deref().map_or(0, |state| state.lobby.len())
    );
    commands
        .entity(*column)
        .despawn_related::<Children>()
        .with_children(|parent| {
            let Some(state) = state.as_deref() else {
                return;
            };
            for (name, record) in state.lobby.iter() {
                parent.spawn((
                    JoinButton(name.clone()),
                    Button,
                    Hovered::default(),
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(ROOM_COLOR),
                    children![Text::new(format!(
                        "{} ({}/{})",
                        name.as_str(),
                        record.members.len(),
                        record.room_size
                    ))],
                ));
            }
        });
}

/// # needed helper: one side-list row per `RoomRoster` entry, rebuilt on roster change
fn rebuild_members(
    mut commands: Commands,
    roster: Option<Res<discovery::RoomRoster>>,
    list: Single<Entity, With<MemberList>>,
) {
    let Some(roster) = roster else {
        return;
    };
    if !roster.is_changed() {
        return;
    }
    commands
        .entity(*list)
        .despawn_related::<Children>()
        .with_children(|rows| {
            for entry in &roster.entries {
                rows.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(ROOM_COLOR),
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: Val::Px(6.0),
                            height: Val::Px(6.0),
                            ..default()
                        },
                        BackgroundColor(match entry.status {
                            discovery::MemberRoomStatus::Connected => MEMBER_CONNECTED_COLOR,
                            discovery::MemberRoomStatus::Connecting => MEMBER_CONNECTING_COLOR,
                        }),
                    ));
                    row.spawn((
                        Text::new(entry.name.clone()),
                        TextFont {
                            font_size: FontSize::Px(11.0),
                            ..default()
                        },
                        TextLayout::no_wrap(),
                        TextColor(BUTTON_LABEL_COLOR),
                    ));
                    if entry.is_me {
                        row.spawn((
                            Text::new("(you)"),
                            TextFont {
                                font_size: FontSize::Px(9.0),
                                ..default()
                            },
                            TextColor(MEMBER_YOU_COLOR),
                        ));
                    }
                });
            }
        });
}

fn on_create(
    buttons: Query<(), (Added<Pressed>, With<NewRoomButton>)>,
    field: Single<&EditableText, With<NameField>>,
    mut commands: MessageWriter<discovery::Command>,
) {
    for () in &buttons {
        let typed = field.value().to_string();
        let name = typed.trim();
        if name.is_empty() {
            tracing::error!("lobby ui create rejected: empty room name");
            continue;
        }
        tracing::info!("lobby ui click create room={name}");
        commands.write(discovery::Command::Create(identity::RoomName::from(name)));
    }
}

/// # needed helper: the Create button is only clickable while the room name is non-empty
fn gate_create(
    input: Single<&EditableText, With<NameField>>,
    button: Single<NewRoomButtonParts, With<NewRoomButton>>,
    mut labels: Query<&mut TextColor>,
    mut commands: Commands,
) {
    let typed = input.into_inner().value().to_string();
    let enabled = !typed.trim().is_empty();
    let (entity, hovered, disabled, mut background, children) = button.into_inner();
    let BackgroundColor(existing) = &mut *background;
    let (color, label_color) = if enabled {
        (BUTTON_COLOR, BUTTON_LABEL_COLOR)
    } else {
        (DISABLED_BUTTON_COLOR, DISABLED_BUTTON_LABEL_COLOR)
    };
    if *existing != color {
        *existing = color;
    }
    let mut label_iter = labels.iter_many_mut(children).matched();
    while let Some(mut label) = label_iter.fetch_next() {
        let TextColor(current) = &mut *label;
        if *current != label_color {
            *current = label_color;
        }
    }
    match (enabled, hovered.is_some()) {
        (true, false) => {
            commands.entity(entity).insert(Hovered(false));
        }
        (false, true) => {
            commands
                .entity(entity)
                .remove::<Hovered>()
                .remove::<Pressed>();
        }
        _ => {}
    }
    match (enabled, disabled) {
        (true, true) => {
            commands.entity(entity).remove::<InteractionDisabled>();
        }
        (false, false) => {
            commands.entity(entity).insert(InteractionDisabled);
        }
        _ => {}
    }
}

fn on_join(
    buttons: Query<&JoinButton, Added<Pressed>>,
    mut commands: MessageWriter<discovery::Command>,
) {
    for button in &buttons {
        tracing::info!("lobby ui click join room={}", button.as_str());
        commands.write(discovery::Command::Join((**button).clone()));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::identity;

    use super::LobbyUi;
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::discovery;
    use freenet_libp2p_bevy_plugin::testing;
    use freenet_libp2p_bevy_plugin::time;

    fn preview_build(app: &mut App) {
        app.add_message::<discovery::Command>();
        app.insert_resource(preview_snapshot());
        app.add_plugins(LobbyUi);
    }

    fn preview_snapshot() -> DiscoveryState {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("alpha"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: std::collections::BTreeMap::new(),
            },
        );
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build()
    }

    const fn preview_noop(_world: &mut World) {}

    fn add_room(world: &mut World) {
        let mut state = world.resource_mut::<DiscoveryState>();
        state.lobby.insert(
            identity::RoomName::from("gamma"),
            discovery::Room {
                room_size: discovery::RoomSize::try_from(4).expect("fixture room size is non-zero"),
                created_at: time::UnixTime::EPOCH,
                members: std::collections::BTreeMap::new(),
            },
        );
    }

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene(
        name: &str,
        timeline: Option<lele_bevy_preview::scene::Timeline>,
    ) -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: name.to_string(),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("one room"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("two rooms"),
                    apply: add_room,
                },
            ],
            timeline,
        }
    }

    #[test]
    fn lobby_ui_png_preview() {
        scene("lobby_ui", None).run(&config()).expect("preview");
    }

    #[test]
    fn lobby_ui_scene_preview() {
        scene("lobby_ui", None).run(&config()).expect("preview");
    }

    #[test]
    fn lobby_ui_mp4_preview() {
        let timeline = lele_bevy_preview::scene::Timeline {
            label: String::from("add_gamma"),
            frames: 12,
            fps: 12,
            apply: add_room,
        };
        scene("lobby_ui", Some(timeline))
            .run(&config())
            .expect("preview");
    }

    fn members_preview_build(app: &mut App) {
        app.add_message::<discovery::Command>();
        app.insert_resource(discovery::RoomRoster::default());
        app.add_plugins(LobbyUi);
    }

    fn room_entry(
        id: &str,
        name: &str,
        status: discovery::MemberRoomStatus,
        is_me: bool,
    ) -> discovery::RoomRosterEntry {
        discovery::RoomRosterEntry {
            id: identity::PeerId::from(id),
            name: String::from(name),
            status,
            is_me,
        }
    }

    fn room_with_members(world: &mut World) {
        *world.resource_mut::<discovery::RoomRoster>() = discovery::RoomRoster {
            own: discovery::OwnRoomStatus::Connected,
            count: 3,
            max: 8,
            entries: vec![
                room_entry(
                    "12D3KooWcarol0Zm9c",
                    "carol",
                    discovery::MemberRoomStatus::Connecting,
                    false,
                ),
                room_entry(
                    "12D3KooWbob000Kd2a",
                    "bob",
                    discovery::MemberRoomStatus::Connected,
                    false,
                ),
                room_entry("me", "alice", discovery::MemberRoomStatus::Connected, true),
            ],
        };
    }

    #[test]
    fn custom_ui_members_ui_png_preview() {
        lele_bevy_preview::scene::Scene {
            name: String::from("custom_ui_members"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: members_preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("no room yet"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("room with members"),
                    apply: room_with_members,
                },
            ],
            timeline: None,
        }
        .run(&config())
        .expect("preview");
    }
}
