use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::discovery;
use core_discovery::state_machine::DiscoveryState;
use discovery::basic::constants::ROOM_COLOR;
use discovery::{HoverGlow, RoomButton, RoomList, RoomRow};
use freenet_libp2p_bevy_core::discovery as core_discovery;

pub fn rebuild_room_list(
    mut commands: Commands,
    state: Option<Res<DiscoveryState>>,
    list: Single<(Entity, Ref<RoomList>)>,
    mut shown: Local<Vec<RoomRow>>,
) {
    let wanted: Vec<RoomRow> = state
        .map(|state| {
            state
                .into_inner()
                .lobby
                .iter()
                .map(|(name, record)| RoomRow {
                    name: name.clone(),
                    members: record.members.len(),
                    room_size: record.room_size,
                })
                .collect()
        })
        .unwrap_or_default();
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
                    Hovered::default(),
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
    use crate::discovery;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_core::identity;

    use super::rebuild_room_list;
    use core_discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_core::discovery as core_discovery;
    use freenet_libp2p_bevy_core::testing;
    use freenet_libp2p_bevy_core::time;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let mut lobby = core_discovery::Lobby::default();
        for name in ["alpha", "beta"] {
            lobby.insert(
                identity::RoomName::from(name),
                core_discovery::Room {
                    room_size: core_discovery::RoomSize::DEFAULT,
                    created_at: time::UnixTime::EPOCH,
                    members: std::collections::BTreeMap::new(),
                },
            );
        }
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build();
        app.insert_resource(state);
        app.world_mut().spawn(discovery::RoomList);
        app.add_systems(Update, rebuild_room_list);
        app.update();
        let mut buttons = app.world_mut().query::<&discovery::RoomButton>();
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

    fn preview_snapshot() -> DiscoveryState {
        let mut lobby = core_discovery::Lobby::default();
        for (name, room_size) in [("alpha", 8), ("beta", 4)] {
            lobby.insert(
                identity::RoomName::from(name),
                core_discovery::Room {
                    room_size: core_discovery::RoomSize::try_from(room_size)
                        .expect("fixture room size is non-zero"),
                    created_at: time::UnixTime::EPOCH,
                    members: std::collections::BTreeMap::new(),
                },
            );
        }
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build()
    }

    fn spawn_list(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            discovery::RoomList,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ));
    }

    const fn preview_noop(_world: &mut World) {}

    fn add_room(world: &mut World) {
        let mut state = world.resource_mut::<DiscoveryState>();
        state.lobby.insert(
            identity::RoomName::from("gamma"),
            core_discovery::Room {
                room_size: core_discovery::RoomSize::try_from(4)
                    .expect("fixture room size is non-zero"),
                created_at: time::UnixTime::EPOCH,
                members: std::collections::BTreeMap::new(),
            },
        );
    }

    #[test]
    fn rebuild_room_list_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
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
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
