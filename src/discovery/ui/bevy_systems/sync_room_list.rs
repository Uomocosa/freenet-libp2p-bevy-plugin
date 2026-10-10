use bevy::prelude::*;

use crate::discovery;
use discovery::ui::{RoomButton, RoomList};

const ROOM_COLOR: Color = Color::srgb(0.16, 0.18, 0.22);
const CURRENT_ROOM_COLOR: Color = Color::srgb(0.18, 0.45, 0.25);

pub fn sync_room_list(
    mut commands: Commands,
    snapshot: Res<discovery::Snapshot>,
    list: Single<Entity, With<RoomList>>,
    mut ready: Local<bool>,
) {
    if *ready && !snapshot.is_changed() {
        return;
    }
    *ready = true;
    let snapshot = snapshot.into_inner();
    let list = list.into_inner();
    let current = snapshot.room.as_ref().map(|room| &room.name);
    commands
        .entity(list)
        .despawn_related::<Children>()
        .with_children(|parent| {
            for (name, record) in &snapshot.lobby {
                let color = if current == Some(name) {
                    CURRENT_ROOM_COLOR
                } else {
                    ROOM_COLOR
                };
                let label = format!(
                    "{} ({}/{})",
                    name.as_str(),
                    record.members.len(),
                    record.capacity
                );
                parent.spawn((
                    RoomButton(name.clone()),
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(color),
                    children![Text::new(label)],
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use crate::net_id;
    use bevy::prelude::*;

    use super::sync_room_list;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let mut snapshot = discovery::Snapshot::default();
        for name in ["alpha", "beta"] {
            snapshot.lobby.insert(
                net_id::RoomName(name.to_string()),
                discovery::RoomEntry {
                    capacity: 8,
                    members: std::collections::BTreeMap::new(),
                },
            );
        }
        app.insert_resource(snapshot);
        app.world_mut().spawn(discovery::ui::RoomList);
        app.add_systems(Update, sync_room_list);
        app.update();
        let mut buttons = app.world_mut().query::<&discovery::ui::RoomButton>();
        let mut names: Vec<String> = buttons
            .iter(app.world())
            .map(|button| button.as_str().to_string())
            .collect();
        names.sort();
        assert_eq!(names, vec!["alpha".to_string(), "beta".to_string()]);
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_snapshot());
        app.add_systems(Startup, spawn_list);
        app.add_systems(Update, sync_room_list);
    }

    fn preview_snapshot() -> discovery::Snapshot {
        let mut snapshot = discovery::Snapshot::default();
        for (name, capacity) in [("alpha", 8_u16), ("beta", 4_u16)] {
            snapshot.lobby.insert(
                net_id::RoomName(name.to_string()),
                discovery::RoomEntry {
                    capacity,
                    members: std::collections::BTreeMap::new(),
                },
            );
        }
        snapshot
    }

    fn spawn_list(mut commands: Commands) {
        commands.spawn((discovery::ui::RoomList, Node::default()));
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

    #[test]
    #[ignore = "headed preview"]
    fn sync_room_list_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::run(
            &lele_bevy_preview::scene::Scene {
                name: String::from("sync_room_list"),
                kind: lele_bevy_preview::scene::Kind::System,
                build: preview_build,
                states: vec![
                    lele_bevy_preview::scene::State {
                        label: String::from("two rooms"),
                        apply: preview_noop,
                    },
                    lele_bevy_preview::scene::State {
                        label: String::from("three rooms"),
                        apply: add_room,
                    },
                ],
                timeline: None,
            },
            &lele_bevy_preview::preview::Config {
                out_dir: out,
                ..Default::default()
            },
            env!("CARGO_PKG_NAME"),
        )
        .expect("preview");
    }
}
