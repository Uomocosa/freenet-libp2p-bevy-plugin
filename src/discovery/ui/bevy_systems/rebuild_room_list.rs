use bevy::prelude::*;

use crate::discovery;
use discovery::ui::basic::constants::ROOM_COLOR;
use discovery::ui::{HoverGlow, RoomButton, RoomList, RoomRow};

pub fn rebuild_room_list(
    mut commands: Commands,
    status: Res<discovery::Status>,
    list: Single<(Entity, Ref<RoomList>)>,
    mut shown: Local<Vec<RoomRow>>,
) {
    let status = status.into_inner();
    let wanted: Vec<RoomRow> = status
        .lobby
        .iter()
        .map(|(name, record)| RoomRow {
            name: name.clone(),
            members: record.members.len(),
            room_size: record.room_size,
        })
        .collect();
    let (list, marker) = list.into_inner();
    if *shown == wanted && !marker.is_added() {
        return;
    }
    *shown = wanted;
    commands
        .entity(list)
        .despawn_related::<Children>()
        .with_children(|parent| {
            for row in &*shown {
                let label = format!(
                    "> {} [{}/{} online]",
                    row.name.as_str(),
                    row.members,
                    row.room_size
                );
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

    use super::rebuild_room_list;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let mut status = discovery::Status::default();
        for name in ["alpha", "beta"] {
            status.lobby.insert(
                identity::RoomName::from(name),
                discovery::LobbyRoom {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at: discovery::UnixTime::from_secs(0),
                    members: std::collections::BTreeMap::new(),
                },
            );
        }
        app.insert_resource(status);
        app.world_mut().spawn(discovery::ui::RoomList);
        app.add_systems(Update, rebuild_room_list);
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
        app.add_systems(Update, rebuild_room_list);
    }

    fn preview_snapshot() -> discovery::Status {
        let mut status = discovery::Status::default();
        for (name, room_size) in [("alpha", 8), ("beta", 4)] {
            status.lobby.insert(
                identity::RoomName::from(name),
                discovery::LobbyRoom {
                    room_size: discovery::RoomSize::try_from(room_size)
                        .expect("fixture room size is non-zero"),
                    created_at: discovery::UnixTime::from_secs(0),
                    members: std::collections::BTreeMap::new(),
                },
            );
        }
        status
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
        let mut status = world.resource_mut::<discovery::Status>();
        status.lobby.insert(
            identity::RoomName::from("gamma"),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::try_from(4).expect("fixture room size is non-zero"),
                created_at: discovery::UnixTime::from_secs(0),
                members: std::collections::BTreeMap::new(),
            },
        );
    }

    #[test]
    fn rebuild_room_list_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::run(
            &lele_bevy_preview::scene::Scene {
                name: String::from("rebuild_room_list"),
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
