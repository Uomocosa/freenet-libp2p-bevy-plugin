use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

use crate::starboard;

pub fn sync_board_visibility(
    status: Res<discovery::Status>,
    mut boards: Query<&mut Visibility, With<starboard::render::BoardSprite>>,
) {
    let status = status.into_inner();
    let wanted = if starboard::room::is_in_room(status) {
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
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::sync_board_visibility;
    use crate::starboard;

    fn status_with(phase: discovery::JoinPhase) -> discovery::Status {
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(0),
                phase,
                members: discovery::Members::new(),
            }),
            ..discovery::Status::default()
        }
    }

    fn visibility_after(status: discovery::Status) -> Visibility {
        let mut app = App::new();
        app.insert_resource(status);
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
            visibility_after(discovery::Status::default()),
            Visibility::Hidden
        );
    }

    fn lobby_status() -> discovery::Status {
        let mut status = discovery::Status {
            freenet_peers: vec![identity::FreenetPeer {
                id: identity::FreenetPeerId::from("gateway"),
                addr: std::net::SocketAddr::from(([127, 0, 0, 1], 31337)),
            }],
            is_lobby_attached: true,
            ..status_with(discovery::JoinPhase::Ready)
        };
        status.lobby.insert(
            identity::RoomName::from("alpha"),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from_secs(0),
                members: BTreeMap::new(),
            },
        );
        status
    }

    fn leave_room(world: &mut World) {
        world.resource_mut::<discovery::Status>().room = None;
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_build(app: &mut App) {
        app.add_message::<discovery::Command>();
        app.insert_resource(lobby_status());
        app.init_resource::<starboard::input::OwnBrush>()
            .init_resource::<starboard::render::CanvasView>()
            .add_plugins(discovery::ui::DefaultUiPlugin)
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
        lele_bevy_preview::run(&scene(), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }
}
