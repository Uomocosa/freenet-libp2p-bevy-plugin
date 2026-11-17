use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::lobby_room::{discovery, spectator};
use freenet_libp2p_bevy_plugin::p2p;

use crate::starboard;

pub fn draw_cursor_labels(
    state: Option<Res<DiscoveryState>>,
    ready: Option<Res<p2p::ReadyChannel>>,
    spectated: Query<&spectator::Spectated<starboard::canvas::Canvas>>,
    cursors: Query<
        (
            &Sprite,
            &starboard::render::Owner,
            &starboard::render::CursorPlacement,
        ),
        With<starboard::render::Cursor>,
    >,
    mut labels: Query<
        (&ChildOf, &mut Text2d, &mut Transform),
        With<starboard::render::CursorLabel>,
    >,
) {
    let state = state.map(Res::into_inner);
    let me = ready
        .map(Res::into_inner)
        .and_then(|ready| ready.ready_rx.borrow().clone())
        .map(|me| me.id);
    let stream_zone = spectated
        .iter()
        .find(|spectated| spectated.slot == spectator::SpectateSlot::Stream)
        .map(|spectated| spectated.zone.clone());
    for (child_of, mut text, mut transform) in &mut labels {
        let Ok((sprite, owner, placement)) = cursors.get(child_of.parent()) else {
            continue;
        };
        let zone = match placement {
            starboard::render::CursorPlacement::InRoom => None,
            starboard::render::CursorPlacement::Stream => stream_zone.as_ref(),
            starboard::render::CursorPlacement::Preview => continue,
        };
        let name = match owner {
            starboard::render::Owner::Me => me.as_ref().map_or_else(
                || String::from(starboard::render::OWN_FALLBACK_NAME),
                |me| {
                    state.map_or_else(
                        || me.as_str().to_string(),
                        |state| starboard::room::member_name(state, zone, me),
                    )
                },
            ),
            starboard::render::Owner::Peer(peer) => state.map_or_else(
                || peer.as_str().to_string(),
                |state| starboard::room::member_name(state, zone, peer),
            ),
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
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::{discovery, spectator};
    use freenet_libp2p_bevy_plugin::time;

    use super::draw_cursor_labels;
    use crate::starboard;

    fn member(peer: &str, name: &str) -> discovery::RoomMember {
        discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from(peer),
                addrs: Vec::new(),
            },
            updated_at: time::UnixTime::EPOCH,
            display_name: Some(identity::Username::from(name)),
            membership: discovery::RoomMembership::Occupant,
        }
    }

    fn status_with(peers: &[(&str, &str)], lobby_peers: &[(&str, &str)]) -> DiscoveryState {
        let mut members = discovery::Members::new();
        for (peer, name) in peers {
            members.insert(identity::PeerId::from(*peer), member(peer, name));
        }
        let mut lobby_members = discovery::Members::new();
        for (peer, name) in lobby_peers {
            lobby_members.insert(identity::PeerId::from(*peer), member(peer, name));
        }
        let lobby = discovery::Lobby::from([(
            discovery::RoomName::from("beta"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: lobby_members,
            },
        )]);
        freenet_libp2p_bevy_plugin::lobby_room::testing::DiscoveryStateBuilder::new(
            identity::PeerId::from("me"),
        )
        .with_lobby(lobby)
        .in_room(discovery::JoinedRoom {
            name: discovery::RoomName::from("alpha"),
            created_at: time::UnixTime::EPOCH,
            phase: discovery::JoinPhase::Ready,
            members,
            how_entered: discovery::HowRoomWasEntered::Created,
        })
        .build()
    }

    fn label_for(app: &mut App, placement: starboard::render::CursorPlacement) -> (Entity, Entity) {
        let cursor = app
            .world_mut()
            .spawn((
                starboard::render::Cursor,
                placement,
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
        (cursor, label)
    }

    fn app() -> App {
        let mut app = App::new();
        app.insert_resource(status_with(
            &[("peer-a", "in-room-ada")],
            &[("peer-a", "stream-ada")],
        ));
        app.add_systems(Update, draw_cursor_labels);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        let (_, label) = label_for(&mut app, starboard::render::CursorPlacement::InRoom);
        app.update();

        let text = app.world().get::<Text2d>(label).expect("label text");
        assert_eq!(text.as_str(), "in-room-ada");
        let transform = app
            .world()
            .get::<Transform>(label)
            .expect("label transform");
        assert!(
            (transform.translation.y - (10.0 + starboard::render::LABEL_GAP_PX)).abs() < 0.000_1
        );
    }

    #[test]
    fn a_stream_label_uses_the_member_name_from_the_lobby_record() {
        let mut app = app();
        app.world_mut().spawn(spectator::Spectated {
            slot: spectator::SpectateSlot::Stream,
            zone: identity::ZoneName::from("beta"),
            state: starboard::canvas::Canvas::default(),
            freshness: spectator::SpectateFreshness::Live,
        });
        let (_, label) = label_for(&mut app, starboard::render::CursorPlacement::Stream);
        app.update();

        let text = app.world().get::<Text2d>(label).expect("label text");
        assert_eq!(text.as_str(), "stream-ada");
        assert_ne!(text.as_str(), "peer-a");
    }

    #[test]
    fn an_in_room_label_keeps_the_joined_room_name() {
        let mut app = app();
        let (_, label) = label_for(&mut app, starboard::render::CursorPlacement::InRoom);
        app.update();

        let text = app.world().get::<Text2d>(label).expect("label text");
        assert_eq!(text.as_str(), "in-room-ada");
        assert_ne!(text.as_str(), "stream-ada");
    }
}
