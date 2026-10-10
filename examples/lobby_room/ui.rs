//! Replacement lobby UI used by the discovery e2e test (`--ui custom`).
//!
//! It shows that the plugin's default UI can be swapped out entirely: this
//! plugin only reads `discovery::Snapshot` and writes `discovery::Command`.
#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;
use bevy::text::EditableText;
use derive_more::Deref;
use freenet_libp2p_bevy_plugin::discovery;
use freenet_libp2p_bevy_plugin::net_id;

const PANEL_COLOR: Color = Color::srgb(0.30, 0.12, 0.34);
const FIELD_COLOR: Color = Color::srgb(0.10, 0.10, 0.12);
const ROOM_COLOR: Color = Color::srgb(0.12, 0.28, 0.38);

pub struct LobbyUi;

impl Plugin for LobbyUi {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (rebuild_list, on_create, on_join));
    }
}

#[derive(Component)]
pub struct NameField;

#[derive(Component)]
pub struct NewRoomButton;

#[derive(Component)]
struct RoomColumn;

#[derive(Component, Deref)]
pub struct JoinButton(pub net_id::RoomName);

fn spawn(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            right: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(PANEL_COLOR),
        children![
            (
                NameField,
                EditableText::new(""),
                Node {
                    width: Val::Px(200.0),
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(FIELD_COLOR),
            ),
            (
                NewRoomButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    ..default()
                },
                children![Text::new("+ New lobby")],
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
    snapshot: Res<discovery::Snapshot>,
    column: Single<Entity, With<RoomColumn>>,
) {
    if !snapshot.is_changed() {
        return;
    }
    tracing::info!("lobby ui rooms={}", snapshot.lobby.len());
    commands
        .entity(*column)
        .despawn_related::<Children>()
        .with_children(|parent| {
            for (name, record) in &snapshot.lobby {
                parent.spawn((
                    JoinButton(name.clone()),
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(ROOM_COLOR),
                    children![Text::new(format!(
                        "> {} [{} online]",
                        name.as_str(),
                        record.members.len()
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
            continue;
        }
        tracing::info!("lobby ui click create room={name}");
        commands.write(discovery::Command::Create(net_id::RoomName(
            name.to_string(),
        )));
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
    use freenet_libp2p_bevy_plugin::net_id;

    use super::{JoinButton, LobbyUi, NameField, NewRoomButton, RoomColumn};
    use freenet_libp2p_bevy_plugin::discovery;

    fn preview_build(app: &mut App) {
        app.add_message::<discovery::Command>();
        app.insert_resource(preview_snapshot());
        app.add_plugins(LobbyUi);
    }

    fn preview_snapshot() -> discovery::Snapshot {
        let mut snapshot = discovery::Snapshot::default();
        snapshot.lobby.insert(
            net_id::RoomName::from("alpha"),
            discovery::RoomEntry {
                capacity: 8,
                members: std::collections::BTreeMap::new(),
            },
        );
        snapshot
    }

    const fn preview_noop(_world: &mut World) {}

    fn add_room(world: &mut World) {
        let mut snapshot = world.resource_mut::<discovery::Snapshot>();
        snapshot.lobby.insert(
            net_id::RoomName::from("gamma"),
            discovery::RoomEntry {
                capacity: 4,
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
    #[ignore = "headed preview"]
    fn lobby_ui_png_preview() {
        let _markers: Option<(NameField, NewRoomButton, RoomColumn, JoinButton)> = None;
        lele_bevy_preview::run(&scene("lobby_ui", None), &config(), env!("CARGO_PKG_NAME"))
            .expect("preview");
    }

    #[test]
    #[ignore = "headed scene"]
    fn lobby_ui_scene_preview() {
        lele_bevy_preview::run(&scene("lobby_ui", None), &config(), env!("CARGO_PKG_NAME"))
            .expect("preview");
    }

    #[test]
    #[ignore = "headed recording"]
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
