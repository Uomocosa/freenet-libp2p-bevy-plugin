use std::collections::HashSet;

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::direct_dial;
use crate::discovery;
use direct_dial::{FriendList, FriendRemoveButton, FriendRow};
use discovery::{BUTTON_COLOR, ROOM_COLOR};
use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
use freenet_libp2p_bevy_core::identity;
use freenet_libp2p_bevy_core::peers;

pub fn rebuild_friend_list(
    mut commands: Commands,
    state: Res<core_direct_dial::state_machine::DialState>,
    connected_peers: Query<&peers::Peer, With<peers::Connected>>,
    list: Single<(Entity, Ref<FriendList>)>,
    mut shown: Local<Vec<(identity::PeerId, bool)>>,
) {
    let state = state.into_inner();
    let connected: HashSet<identity::PeerId> = connected_peers
        .iter()
        .map(|peer| (**peer).clone())
        .collect();
    let wanted: Vec<(identity::PeerId, bool)> = state
        .entries
        .iter()
        .map(|entry| (entry.peer.id.clone(), connected.contains(&entry.peer.id)))
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
            for (id, is_connected) in &*shown {
                let status = if *is_connected {
                    "connected"
                } else {
                    "offline"
                };
                let label = format!("> {} [{status}]", id.as_str());
                parent.spawn((
                    FriendRow(id.clone()),
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(ROOM_COLOR),
                    children![
                        Text::new(label),
                        (
                            FriendRemoveButton(id.clone()),
                            Button,
                            Hovered::default(),
                            Node {
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(BUTTON_COLOR),
                            children![Text::new("Remove")],
                        )
                    ],
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::rebuild_friend_list;
    use crate::direct_dial;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;
    use freenet_libp2p_bevy_core::peers;
    use freenet_libp2p_bevy_core::time;

    fn state_with(names: &[&str]) -> core_direct_dial::state_machine::DialState {
        let mut book = core_direct_dial::PeerBook::default();
        for name in names {
            book.add_friend(identity::Peer {
                id: identity::PeerId::from(*name),
                addrs: Vec::new(),
            });
        }
        core_direct_dial::state_machine::DialState::from_peer_book(
            &book,
            time::UnixTime::from_secs(10),
        )
    }

    fn app_with(state: core_direct_dial::state_machine::DialState, connected: &[&str]) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(state);
        for name in connected {
            app.world_mut().spawn((
                peers::Peer::from(identity::PeerId::from(*name)),
                peers::Connected,
            ));
        }
        app.world_mut().spawn(direct_dial::FriendList);
        app.add_systems(Update, rebuild_friend_list);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(state_with(&["alice", "bob"]), &[]);
        app.update();
        let mut rows = app.world_mut().query::<&direct_dial::FriendRow>();
        assert_eq!(rows.iter(app.world()).count(), 2);
    }

    #[test]
    fn row_flips_connected_and_offline_on_events() {
        let mut app = app_with(state_with(&["alice"]), &[]);
        app.update();
        let label = |world: &mut World| {
            world
                .query_filtered::<&Text, Without<direct_dial::OwnAddressText>>()
                .iter(world)
                .map(|text| text.as_str().to_string())
                .collect::<Vec<_>>()
        };
        assert!(
            label(app.world_mut())
                .iter()
                .any(|text| text.contains("offline"))
        );
        let alice = app
            .world_mut()
            .spawn((
                peers::Peer::from(identity::PeerId::from("alice")),
                peers::Connected,
            ))
            .id();
        app.update();
        assert!(
            label(app.world_mut())
                .iter()
                .any(|text| text.contains("connected"))
        );
        app.world_mut()
            .entity_mut(alice)
            .remove::<peers::Connected>();
        app.update();
        assert!(
            label(app.world_mut())
                .iter()
                .any(|text| text.contains("offline"))
        );
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(state_with(&["alice"]));
        app.add_systems(Startup, spawn_list);
        app.add_systems(Update, rebuild_friend_list);
    }

    fn spawn_list(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            direct_dial::FriendList,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ));
    }

    const fn preview_noop(_world: &mut World) {}

    fn connect_alice(world: &mut World) {
        world.spawn((
            peers::Peer::from(identity::PeerId::from("alice")),
            peers::Connected,
        ));
    }

    #[test]
    fn rebuild_friend_list_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_friend_list"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("one friend offline"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("friend connected"),
                    apply: connect_alice,
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
