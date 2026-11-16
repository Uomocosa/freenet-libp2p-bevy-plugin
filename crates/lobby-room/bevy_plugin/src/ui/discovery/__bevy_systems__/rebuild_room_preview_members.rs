use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    BUTTON_LABEL_COLOR, CONNECTING_TEXT_COLOR, PREVIEW_MEMBER_FONT_SIZE, RoomPreviewViewport,
    SelectedRoom,
};

pub fn rebuild_room_preview_members(
    selection: Res<SelectedRoom>,
    state: Option<Res<core_discovery::state_machine::DiscoveryState>>,
    mut commands: Commands,
    viewport: Option<Single<Entity, With<RoomPreviewViewport>>>,
    mut shown: Local<Option<Vec<String>>>,
) {
    let Some(viewport) = viewport else {
        return;
    };
    let selection = selection.into_inner();
    let Some(name) = selection.room.clone() else {
        return;
    };
    let state = state.map(Res::into_inner);
    let members: Vec<String> = state
        .and_then(|state| state.lobby.get(&name))
        .map(|room| {
            room.members
                .values()
                .filter(|member| member.membership == core_discovery::RoomMembership::Occupant)
                .map(|member| {
                    member
                        .display_name
                        .as_ref()
                        .map_or_else(|| member.peer.id.as_str().to_string(), ToString::to_string)
                })
                .collect()
        })
        .unwrap_or_default();
    if shown.as_ref() == Some(&members) {
        return;
    }
    *shown = Some(members.clone());
    commands
        .entity(viewport.into_inner())
        .despawn_related::<Children>()
        .with_children(|viewport| {
            if members.is_empty() {
                viewport.spawn((
                    Text::new("Empty room"),
                    TextFont {
                        font_size: FontSize::Px(PREVIEW_MEMBER_FONT_SIZE),
                        ..default()
                    },
                    TextColor(CONNECTING_TEXT_COLOR),
                ));
            }
            for member in &members {
                viewport.spawn((
                    Text::new(member.clone()),
                    TextFont {
                        font_size: FontSize::Px(PREVIEW_MEMBER_FONT_SIZE),
                        ..default()
                    },
                    TextColor(BUTTON_LABEL_COLOR),
                    TextLayout::no_wrap(),
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::testing;
    use crate::ui;
    use bevy::prelude::*;

    use super::rebuild_room_preview_members;

    fn state_with(
        room: &str,
        members: &[(&str, core_discovery::RoomMembership)],
    ) -> core_discovery::state_machine::DiscoveryState {
        let mut lobby = core_discovery::Lobby::default();
        lobby.insert(
            core_discovery::RoomName::from(room),
            core_discovery::Room {
                room_size: core_discovery::RoomSize::try_from(4).expect("4"),
                created_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                members: members
                    .iter()
                    .map(|(name, membership)| {
                        (
                            freenet_libp2p_bevy_core::identity::PeerId::from(*name),
                            core_discovery::RoomMember {
                                peer: freenet_libp2p_bevy_core::identity::Peer {
                                    id: freenet_libp2p_bevy_core::identity::PeerId::from(*name),
                                    addrs: Vec::new(),
                                },
                                display_name: Some(
                                    freenet_libp2p_bevy_core::identity::Username::from(*name),
                                ),
                                updated_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                                membership: *membership,
                            },
                        )
                    })
                    .collect(),
            },
        );
        testing::DiscoveryStateBuilder::new(freenet_libp2p_bevy_core::identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build()
    }

    fn app_with(room: &str, state: core_discovery::state_machine::DiscoveryState) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ui::discovery::SelectedRoom {
            room: Some(core_discovery::RoomName::from(room)),
            is_locked: true,
        });
        app.insert_resource(state);
        app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_preview);
        app.update();
        app.add_systems(Update, rebuild_room_preview_members);
        app
    }

    fn viewport_texts(app: &mut App) -> Vec<String> {
        let mut viewports = app
            .world_mut()
            .query_filtered::<Entity, With<ui::discovery::RoomPreviewViewport>>();
        let viewport = viewports.single(app.world()).expect("viewport");
        let children = app
            .world()
            .entity(viewport)
            .get::<Children>()
            .expect("viewport children");
        children
            .iter()
            .map(|child| {
                app.world()
                    .get::<Text>(child)
                    .expect("text")
                    .as_str()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(
            "alpha",
            state_with(
                "alpha",
                &[("alice", core_discovery::RoomMembership::Occupant)],
            ),
        );
        app.update();
        assert_eq!(viewport_texts(&mut app), vec!["alice".to_string()]);
    }

    #[test]
    fn only_occupants_are_listed() {
        let mut app = app_with(
            "alpha",
            state_with(
                "alpha",
                &[
                    ("alice", core_discovery::RoomMembership::Occupant),
                    ("bob", core_discovery::RoomMembership::HistoryHolder),
                    ("carol", core_discovery::RoomMembership::Departed),
                ],
            ),
        );
        app.update();
        assert_eq!(viewport_texts(&mut app), vec!["alice".to_string()]);
    }

    #[test]
    fn an_empty_room_shows_the_empty_room_fallback() {
        let mut app = app_with("beta", state_with("beta", &[]));
        app.update();
        assert_eq!(viewport_texts(&mut app), vec!["Empty room".to_string()]);
    }

    fn preview_row(
        name: &str,
        occupancy: core_discovery::RoomOccupancy,
    ) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants: 0,
            room_size: core_discovery::RoomSize::try_from(4).expect("4"),
            occupancy,
            friends: 0,
        }
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(core_discovery::LobbyRows(vec![
            preview_row("alpha", core_discovery::RoomOccupancy::HasSpace),
            preview_row("beta", core_discovery::RoomOccupancy::Empty),
        ]));
        app.insert_resource(state_with(
            "alpha",
            &[
                ("alice", core_discovery::RoomMembership::Occupant),
                ("bob", core_discovery::RoomMembership::Occupant),
            ],
        ));
        app.insert_resource(ui::discovery::SelectedRoom {
            room: Some(core_discovery::RoomName::from("alpha")),
            is_locked: true,
        });
        app.add_systems(Startup, ui::discovery::bevy_systems::spawn_room_preview);
        app.add_systems(
            Update,
            (
                ui::discovery::bevy_systems::refresh_room_preview,
                ui::discovery::bevy_systems::rebuild_room_preview_members,
            ),
        );
    }

    fn select_room(world: &mut World, room: &str) {
        let mut selection = world.resource_mut::<ui::discovery::SelectedRoom>();
        selection.room = Some(core_discovery::RoomName::from(room));
        selection.is_locked = true;
    }

    fn select_alpha(world: &mut World) {
        select_room(world, "alpha");
    }

    fn select_beta(world: &mut World) {
        select_room(world, "beta");
    }

    #[test]
    fn rebuild_room_preview_members_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_room_preview_members"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("two occupants"),
                    apply: select_alpha,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("empty room"),
                    apply: select_beta,
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
