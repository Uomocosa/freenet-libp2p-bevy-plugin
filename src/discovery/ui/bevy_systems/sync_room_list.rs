use bevy::prelude::*;

use crate::discovery;
use discovery::ui::basic::constants::ROOM_COLOR;
use discovery::ui::{HoverGlow, RoomButton, RoomDetails, RoomList};

pub fn sync_room_list(
    mut commands: Commands,
    snapshot: Res<discovery::Snapshot>,
    list: Single<Entity, With<RoomList>>,
    mut shown: Local<Vec<RoomDetails>>,
) {
    let snapshot = snapshot.into_inner();
    let wanted: Vec<RoomDetails> = snapshot
        .lobby
        .iter()
        .map(|(name, record)| RoomDetails {
            name: name.clone(),
            members: record.members.len(),
            room_size: record.room_size,
        })
        .collect();
    if *shown == wanted {
        return;
    }
    *shown = wanted;
    let list = list.into_inner();
    commands
        .entity(list)
        .despawn_related::<Children>()
        .with_children(|parent| {
            for row in &*shown {
                let label = format!("> {} [{} online]", row.name.as_str(), row.members);
                parent.spawn((
                    RoomButton(row.name.clone()),
                    Button,
                    HoverGlow::default(),
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(ROOM_COLOR),
                    children![Text::new(label)],
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use bevy::prelude::*;

    use super::sync_room_list;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let mut snapshot = discovery::Snapshot::default();
        for name in ["alpha", "beta"] {
            snapshot.lobby.insert(
                identity::RoomName::from(name),
                discovery::Room {
                    room_size: 8,
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
        for (name, room_size) in [("alpha", 8_u16), ("beta", 4_u16)] {
            snapshot.lobby.insert(
                identity::RoomName::from(name),
                discovery::Room {
                    room_size,
                    members: std::collections::BTreeMap::new(),
                },
            );
        }
        snapshot
    }

    fn spawn_list(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            discovery::ui::RoomList,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ));
    }

    const fn preview_noop(_world: &mut World) {}

    fn add_room(world: &mut World) {
        let mut snapshot = world.resource_mut::<discovery::Snapshot>();
        snapshot.lobby.insert(
            identity::RoomName::from("gamma"),
            discovery::Room {
                room_size: 4,
                members: std::collections::BTreeMap::new(),
            },
        );
    }

    #[test]
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
