use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::discovery;

use crate::starboard;

pub fn sync_board_visibility(
    state: Option<Res<DiscoveryState>>,
    mut boards: Query<&mut Visibility, With<starboard::render::BoardSprite>>,
) {
    let wanted = if state.is_some_and(|state| starboard::room::is_in_room(state.into_inner())) {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut boards {
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bevy::prelude::*;
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::time;
    use freenet_libp2p_bevy_plugin::{discovery, identity, testing, ui};

    use super::sync_board_visibility;
    use crate::starboard;

    fn status_with(phase: discovery::JoinPhase) -> DiscoveryState {
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .in_room(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: time::UnixTime::EPOCH,
                phase,
                members: discovery::Members::new(),
                how_entered: discovery::HowRoomWasEntered::Created,
            })
            .build()
    }

    fn visibility_after(state: DiscoveryState) -> Visibility {
        let mut app = App::new();
        app.insert_resource(state);
        app.world_mut().spawn((
            starboard::render::BoardSprite,
            Sprite::default(),
            Visibility::Visible,
        ));
        app.add_systems(Update, sync_board_visibility);
        app.update();
        let mut boards = app
            .world_mut()
            .query_filtered::<&Visibility, With<starboard::render::BoardSprite>>();
        *boards.single(app.world()).expect("one board sprite")
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            visibility_after(status_with(discovery::JoinPhase::Ready)),
            Visibility::Visible
        );
        assert_eq!(
            visibility_after(
                testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build()
            ),
            Visibility::Hidden
        );
    }

    fn lobby_status() -> DiscoveryState {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("alpha"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: BTreeMap::new(),
            },
        );
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_freenet_peers(vec![freenet_libp2p_bevy_plugin::freenet::FreenetPeer {
                id: freenet_libp2p_bevy_plugin::freenet::FreenetPeerId::from("gateway"),
                addr: std::net::SocketAddr::from(([127, 0, 0, 1], 31337)),
            }])
            .lobby_fetched()
            .with_lobby(lobby)
            .in_room(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: time::UnixTime::EPOCH,
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
                how_entered: discovery::HowRoomWasEntered::Created,
            })
            .build()
    }

    fn leave_room(world: &mut World) {
        world.resource_mut::<DiscoveryState>().room = None;
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_build(app: &mut App) {
        app.add_message::<discovery::Command>();
        app.insert_resource(lobby_status());
        app.init_resource::<starboard::input::OwnBrush>()
            .init_resource::<starboard::render::CanvasView>()
            .add_plugins(ui::discovery::DefaultUiPlugin)
            .add_systems(Startup, starboard::render::bevy_systems::spawn_board)
            .add_systems(
                Update,
                (
                    starboard::render::bevy_systems::fit_board,
                    sync_board_visibility,
                ),
            );
    }

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene() -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("starboard_board_visibility"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("in a room"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("left the room"),
                    apply: leave_room,
                },
            ],
            timeline: None,
        }
    }

    #[test]
    fn sync_board_visibility_ui_png_preview() {
        scene().run(&config()).expect("preview");
    }
}
