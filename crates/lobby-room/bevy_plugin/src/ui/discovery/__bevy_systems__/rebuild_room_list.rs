use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

pub fn rebuild_room_list(
    mut commands: Commands,
    rows: Option<Res<core_discovery::LobbyRows>>,
    list: Single<(Entity, Ref<ui::discovery::RoomList>)>,
    mut shown: Local<Vec<core_discovery::LobbyRow>>,
    mut machines: Query<(
        &ui::discovery::RoomButton,
        &mut ui::roll::RollMachine,
        &ui::discovery::RoomRowParts,
    )>,
    mut texts: Query<&mut Text>,
    mut backgrounds: Query<&mut BackgroundColor>,
) {
    let wanted: Vec<core_discovery::LobbyRow> =
        rows.map(|rows| (**rows).clone()).unwrap_or_default();
    let (list, marker) = list.into_inner();
    if *shown == wanted && !marker.is_added() {
        return;
    }
    let do_names_match = !marker.is_added()
        && shown.len() == wanted.len()
        && shown
            .iter()
            .zip(&wanted)
            .all(|(old, new)| old.name == new.name);
    if do_names_match {
        for row in &wanted {
            ui::discovery::bevy_systems::update_room_row(
                row,
                &mut machines,
                &mut texts,
                &mut backgrounds,
            );
        }
        *shown = wanted;
        return;
    }
    *shown = wanted;
    commands
        .entity(list)
        .despawn_related::<Children>()
        .with_children(|rooms| {
            for row in &*shown {
                ui::discovery::bevy_systems::spawn_room_row(&mut rooms.reborrow(), row);
            }
        });
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::testing;
    use crate::ui;
    use std::collections::BTreeMap;

    use bevy::prelude::*;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;

    use super::rebuild_room_list;

    const ROOMS: [(&str, u16, &[&str]); 4] = [
        ("alpha", 8, &["alice", "bob", "carol"]),
        ("very-long-room-name-that-never-fits", 4, &["dave"]),
        ("beta", 8, &[]),
        ("gamma", 4, &["erin", "frank", "grace", "henry"]),
    ];

    #[test]
    fn rebuild_room_list_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_room_list"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                let member = |peer: &str| {
                    (
                        identity::PeerId::from(peer),
                        core_discovery::RoomMember {
                            peer: identity::Peer {
                                id: identity::PeerId::from(peer),
                                addrs: Vec::new(),
                            },
                            display_name: None,
                            updated_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                            membership: core_discovery::RoomMembership::Occupant,
                        },
                    )
                };
                let mut lobby = core_discovery::Lobby::default();
                for (name, size, members) in ROOMS {
                    lobby.insert(
                        core_discovery::RoomName::from(name),
                        core_discovery::Room {
                            room_size: core_discovery::RoomSize::try_from(size)
                                .expect("fixture room size is non-zero"),
                            created_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                            members: members
                                .iter()
                                .copied()
                                .map(member)
                                .collect::<BTreeMap<_, _>>(),
                        },
                    );
                }
                let mut book = core_direct_dial::PeerBook::default();
                for friend in ["alice", "bob"] {
                    assert!(book.add_friend(identity::Peer {
                        id: identity::PeerId::from(friend),
                        addrs: Vec::new(),
                    }));
                }
                app.insert_resource(
                    testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                        .with_lobby(lobby)
                        .build(),
                );
                app.insert_resource(core_direct_dial::state_machine::DialState::from_peer_book(
                    &book,
                    freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                ));
                app.init_resource::<core_discovery::LobbyRows>();
                app.init_resource::<core_discovery::RoomFriendNames>();
                app.add_systems(
                    Startup,
                    (
                        ui::discovery::bevy_systems::spawn_root,
                        lele_bevy_preview::scene::bevy_systems::center_roots,
                    )
                        .chain(),
                );
                app.add_systems(
                    Update,
                    (
                        core_discovery::bevy_systems::rebuild_lobby_rows,
                        rebuild_room_list,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("four rooms sorted"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("lobby cleared"),
                    apply: |world: &mut World| {
                        world.remove_resource::<core_discovery::state_machine::DiscoveryState>();
                    },
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
