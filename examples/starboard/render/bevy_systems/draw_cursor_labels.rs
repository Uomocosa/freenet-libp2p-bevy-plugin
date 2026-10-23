use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{discovery, p2p};

use crate::starboard;

pub fn draw_cursor_labels(
    status: Res<discovery::Status>,
    ready: Option<Res<p2p::ReadyChannel>>,
    cursors: Query<(&Sprite, &starboard::render::Owner), With<starboard::render::Cursor>>,
    mut labels: Query<
        (&ChildOf, &mut Text2d, &mut Transform),
        With<starboard::render::CursorLabel>,
    >,
) {
    let status = status.into_inner();
    let me = ready
        .map(Res::into_inner)
        .and_then(|ready| ready.ready_rx.borrow().clone())
        .map(|me| me.id);
    for (child_of, mut text, mut transform) in &mut labels {
        let Ok((sprite, owner)) = cursors.get(child_of.parent()) else {
            continue;
        };
        let name = match owner {
            starboard::render::Owner::Me => me.as_ref().map_or_else(
                || String::from(starboard::render::OWN_FALLBACK_NAME),
                |me| starboard::room::member_name(status, me),
            ),
            starboard::render::Owner::Peer(peer) => starboard::room::member_name(status, peer),
        };
        if **text != name {
            **text = name;
        }
        let half = sprite.custom_size.map_or(0.0, |size| size.y * 0.5);
        transform.translation = Vec3::new(
            0.0,
            half + starboard::render::LABEL_GAP_PX,
            starboard::render::LABEL_Z,
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::draw_cursor_labels;
    use crate::starboard;

    fn status_with(peers: &[(&str, &str)]) -> discovery::Status {
        let mut members = discovery::Members::new();
        for (peer, name) in peers {
            members.insert(
                identity::PeerId::from(*peer),
                discovery::RoomMember {
                    peer: identity::Peer {
                        id: identity::PeerId::from(*peer),
                        addrs: Vec::new(),
                    },
                    updated_at: discovery::UnixTime::from(0),
                    display_name: Some(identity::Username::from(*name)),
                    membership: discovery::RoomMembership::Occupant,
                },
            );
        }
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from(0),
                phase: discovery::JoinPhase::Ready,
                members,
            }),
            ..discovery::Status::default()
        }
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(status_with(&[("peer-a", "user0"), ("peer-b", "user1")]));
        app.init_resource::<starboard::render::CanvasView>()
            .init_resource::<starboard::input::OwnBrush>()
            .init_resource::<ButtonInput<MouseButton>>()
            .insert_resource(starboard::input::CursorCell(Some(
                starboard::canvas::CellPos { x: 512, y: 700 },
            )))
            .init_resource::<starboard::room::RemoteCursors>()
            .init_resource::<starboard::room::RoomBrushes>()
            .add_systems(Startup, starboard::render::bevy_systems::spawn_board)
            .add_systems(
                Update,
                (
                    starboard::render::bevy_systems::fit_board,
                    starboard::render::bevy_systems::follow_brush,
                    starboard::render::bevy_systems::repaint_own_brush_outline,
                    starboard::render::bevy_systems::draw_remote_cursors,
                    draw_cursor_labels,
                )
                    .chain(),
            );
    }

    const fn preview_own_only(_world: &mut World) {}

    fn preview_with_peers(world: &mut World) {
        let peers = [
            identity::PeerId::from("peer-a"),
            identity::PeerId::from("peer-b"),
        ];
        world.resource_mut::<starboard::room::RoomBrushes>().0 = starboard::identity::room_brushes(
            peers.iter(),
            discovery::UnixTime::from(0),
            discovery::UnixTime::from(1),
        );
        for (peer, x) in peers.into_iter().zip([300_u16, 700]) {
            world
                .resource_mut::<starboard::room::RemoteCursors>()
                .insert(
                    peer,
                    starboard::room::CursorUpdate {
                        cell: starboard::canvas::CellPos { x, y: 300 },
                        size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
                    },
                );
        }
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
            name: String::from("starboard_draw_cursor_labels"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("own cursor only"),
                    apply: preview_own_only,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("two peers"),
                    apply: preview_with_peers,
                },
            ],
            timeline: None,
        }
    }

    #[test]
    fn draw_cursor_labels_ui_png_preview() {
        scene()
            .run(&config(), env!("CARGO_PKG_NAME"))
            .expect("preview");
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(status_with(&[("peer-a", "ada")]));
        let cursor = app
            .world_mut()
            .spawn((
                starboard::render::Cursor,
                starboard::render::Owner::Peer(identity::PeerId::from("peer-a")),
                Sprite {
                    custom_size: Some(Vec2::splat(20.0)),
                    ..default()
                },
            ))
            .id();
        let label = app
            .world_mut()
            .spawn((
                starboard::render::cursor_label(Color::WHITE),
                ChildOf(cursor),
            ))
            .id();
        app.add_systems(Update, draw_cursor_labels);
        app.update();

        let text = app.world().get::<Text2d>(label).expect("label text");
        assert_eq!(text.as_str(), "ada");
        let transform = app
            .world()
            .get::<Transform>(label)
            .expect("label transform");
        assert!(
            (transform.translation.y - (10.0 + starboard::render::LABEL_GAP_PX)).abs() < 0.000_1
        );
    }
}
