//! Replacement lobby UI used by the discovery e2e test (`--ui custom`).
//!
//! It shows that the plugin's default UI can be swapped out entirely: this
//! plugin only reads `discovery::Status` and writes `discovery::Command`.
#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use derive_more::Deref;
use freenet_libp2p_bevy_plugin::discovery;
use freenet_libp2p_bevy_plugin::discovery::ui::basic::constants::{
    BUTTON_COLOR, BUTTON_LABEL_COLOR, CARET_COLOR, DISABLED_BUTTON_COLOR,
    DISABLED_BUTTON_LABEL_COLOR,
};
use freenet_libp2p_bevy_plugin::identity;

const FIELD_COLOR: Color = Color::srgb(0.10, 0.11, 0.14);
const ROOM_COLOR: Color = Color::srgb(0.16, 0.18, 0.22);

pub struct LobbyUi;

impl Plugin for LobbyUi {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (rebuild_list, gate_create, on_create, on_join));
    }
}

#[derive(Component)]
pub struct NameField;

#[derive(Component)]
pub struct NewRoomButton;

#[derive(Component)]
struct RoomColumn;

#[derive(Component, Deref)]
pub struct JoinButton(pub identity::RoomName);

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
}

fn rebuild_list(
    mut commands: Commands,
    status: Res<discovery::Status>,
    column: Single<Entity, With<RoomColumn>>,
) {
    if !status.is_changed() {
        return;
    }
    tracing::info!("lobby ui rooms={}", status.lobby.len());
    commands
        .entity(*column)
        .despawn_related::<Children>()
        .with_children(|parent| {
            for (name, record) in status.lobby.iter() {
                parent.spawn((
                    JoinButton(name.clone()),
                    Button,
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

fn on_create(
    buttons: Query<&Interaction, (Changed<Interaction>, With<NewRoomButton>)>,
    field: Single<&EditableText, With<NameField>>,
    mut commands: MessageWriter<discovery::Command>,
) {
    for interaction in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
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
    button: Single<
        (
            Entity,
            Option<&Interaction>,
            &mut BackgroundColor,
            &Children,
        ),
        With<NewRoomButton>,
    >,
    mut labels: Query<&mut TextColor>,
    mut commands: Commands,
) {
    let typed = input.into_inner().value().to_string();
    let enabled = !typed.trim().is_empty();
    let (entity, interaction, mut background, children) = button.into_inner();
    let BackgroundColor(existing) = &mut *background;
    let (color, label_color) = if enabled {
        (BUTTON_COLOR, BUTTON_LABEL_COLOR)
    } else {
        (DISABLED_BUTTON_COLOR, DISABLED_BUTTON_LABEL_COLOR)
    };
    if *existing != color {
        *existing = color;
    }
    let mut label_iter = labels.iter_many_mut(children);
    while let Some(mut label) = label_iter.fetch_next() {
        let TextColor(current) = &mut *label;
        if *current != label_color {
            *current = label_color;
        }
    }
    match (enabled, interaction.is_some()) {
        (true, false) => {
            commands.entity(entity).insert(Interaction::None);
        }
        (false, true) => {
            commands.entity(entity).remove::<Interaction>();
        }
        _ => {}
    }
}

fn on_join(
    buttons: Query<(&Interaction, &JoinButton), Changed<Interaction>>,
    mut commands: MessageWriter<discovery::Command>,
) {
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed {
            tracing::info!("lobby ui click join room={}", button.as_str());
            commands.write(discovery::Command::Join((**button).clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::identity;

    use super::{JoinButton, LobbyUi, NameField, NewRoomButton, RoomColumn};
    use freenet_libp2p_bevy_plugin::discovery;

    fn preview_build(app: &mut App) {
        app.add_message::<discovery::Command>();
        app.insert_resource(preview_snapshot());
        app.add_plugins(LobbyUi);
    }

    fn preview_snapshot() -> discovery::Status {
        let mut status = discovery::Status::default();
        status.lobby.insert(
            identity::RoomName::from("alpha"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from_secs(0),
                members: std::collections::BTreeMap::new(),
            },
        );
        status
    }

    const fn preview_noop(_world: &mut World) {}

    fn add_room(world: &mut World) {
        let mut status = world.resource_mut::<discovery::Status>();
        status.lobby.insert(
            identity::RoomName::from("gamma"),
            discovery::Room {
                room_size: discovery::RoomSize::try_from(4).expect("fixture room size is non-zero"),
                created_at: discovery::UnixTime::from_secs(0),
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
        let _markers: Option<(NameField, NewRoomButton, RoomColumn, JoinButton)> = None;
        lele_bevy_preview::run(&scene("lobby_ui", None), &config(), env!("CARGO_PKG_NAME"))
            .expect("preview");
    }

    #[test]
    fn lobby_ui_scene_preview() {
        lele_bevy_preview::run(&scene("lobby_ui", None), &config(), env!("CARGO_PKG_NAME"))
            .expect("preview");
    }

    #[test]
    fn lobby_ui_mp4_preview() {
        let timeline = lele_bevy_preview::scene::Timeline {
            label: String::from("add_gamma"),
            frames: 12,
            fps: 12,
            apply: add_room,
        };
        lele_bevy_preview::run(
            &scene("lobby_ui", Some(timeline)),
            &config(),
            env!("CARGO_PKG_NAME"),
        )
        .expect("preview");
    }
}
