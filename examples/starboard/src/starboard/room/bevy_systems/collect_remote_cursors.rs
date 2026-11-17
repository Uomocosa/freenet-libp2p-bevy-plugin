use std::collections::BTreeSet;

use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::lobby_room::{discovery, spectator};
use freenet_libp2p_bevy_plugin::{identity, p2p};

use crate::starboard;

pub fn collect_remote_cursors(
    status: Option<Res<DiscoveryState>>,
    ready: Option<Res<p2p::ReadyChannel>>,
    events: Option<ResMut<p2p::PendingEvents<starboard::room::CursorUpdate>>>,
    spectated: Option<Res<spectator::Spectated<starboard::canvas::Canvas>>>,
    mut cursors: ResMut<starboard::room::RemoteCursors>,
    mut preview: ResMut<starboard::preview::PreviewCursors>,
    mut shown_zone: Local<Option<identity::ZoneName>>,
) {
    let status = status.map(Res::into_inner);
    let own = ready.and_then(|ready| ready.ready_rx.borrow().clone().map(|peer| peer.id));
    let own_zone = status
        .as_ref()
        .and_then(|status| status.room.as_ref())
        .map(|room| identity::ZoneName::clone(&room.name));
    let zone = spectated.map(|spectated| spectated.zone.clone());
    if shown_zone.as_ref() != zone.as_ref() {
        preview.clear();
        (*shown_zone).clone_from(&zone);
    }
    if let Some(mut events) = events {
        for event in events.take_all() {
            if let p2p::Event::ZoneMessage {
                zone: message_zone,
                from,
                payload,
            } = event
            {
                if Some(&from) == own.as_ref() {
                    continue;
                }
                if Some(&message_zone) == zone.as_ref() {
                    preview.insert(from, payload);
                } else if Some(&message_zone) == own_zone.as_ref() {
                    cursors.insert(from, payload);
                }
            }
        }
    }
    let members: BTreeSet<&identity::PeerId> = status
        .as_ref()
        .and_then(|status| status.room.as_ref())
        .map_or_else(BTreeSet::new, |room| room.members.keys().collect());
    cursors.retain(|peer, _| members.contains(peer));
    let preview_members: BTreeSet<&identity::PeerId> = zone
        .as_ref()
        .and_then(|zone| {
            status.and_then(|status| status.lobby.get(&discovery::RoomName::from(zone.clone())))
        })
        .map_or_else(BTreeSet::new, |room| room.members.keys().collect());
    preview.retain(|peer, _| preview_members.contains(peer));
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bevy::prelude::*;
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::lobby_room::{discovery, spectator};
    use freenet_libp2p_bevy_plugin::time;
    use freenet_libp2p_bevy_plugin::{identity, p2p};

    use super::collect_remote_cursors;
    use crate::starboard;

    fn member(peer: &str) -> discovery::RoomMember {
        discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from(peer),
                addrs: Vec::new(),
            },
            updated_at: time::UnixTime::EPOCH,
            display_name: None,
            membership: discovery::RoomMembership::Occupant,
        }
    }

    fn members(peers: &[&str]) -> discovery::Members {
        let mut members = discovery::Members::new();
        for peer in peers {
            members.insert(identity::PeerId::from(*peer), member(peer));
        }
        members
    }

    fn joined_status(peers: &[&str]) -> DiscoveryState {
        freenet_libp2p_bevy_plugin::lobby_room::testing::DiscoveryStateBuilder::new(
            identity::PeerId::from("me"),
        )
        .in_room(discovery::JoinedRoom {
            name: discovery::RoomName::from("alpha"),
            created_at: time::UnixTime::EPOCH,
            phase: discovery::JoinPhase::Ready,
            members: members(peers),
            how_entered: discovery::HowRoomWasEntered::Created,
        })
        .build()
    }

    fn lobby_status(room: &str, peers: &[&str]) -> DiscoveryState {
        let room_members: BTreeMap<identity::PeerId, discovery::RoomMember> = peers
            .iter()
            .map(|peer| (identity::PeerId::from(*peer), member(peer)))
            .collect();
        let lobby = discovery::Lobby::from([(
            discovery::RoomName::from(room),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: room_members,
            },
        )]);
        freenet_libp2p_bevy_plugin::lobby_room::testing::DiscoveryStateBuilder::new(
            identity::PeerId::from("me"),
        )
        .with_lobby(lobby)
        .build()
    }

    fn cursor(x: u16) -> starboard::room::CursorUpdate {
        starboard::room::CursorUpdate {
            cell: starboard::canvas::CellPos { x, y: 0 },
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
        }
    }

    fn app(status: DiscoveryState) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(status);
        app.init_resource::<p2p::ReadyChannel>();
        app.init_resource::<p2p::PendingEvents<starboard::room::CursorUpdate>>();
        app.init_resource::<starboard::room::RemoteCursors>();
        app.init_resource::<starboard::preview::PreviewCursors>();
        app.add_systems(Update, collect_remote_cursors);
        let _ = app
            .world()
            .resource::<p2p::ReadyChannel>()
            .ready_tx
            .send(Some(identity::Peer {
                id: identity::PeerId::from("peer-a"),
                addrs: Vec::new(),
            }));
        app
    }

    fn zone_message(app: &mut App, zone: &str, from: &str, x: u16) {
        app.world_mut()
            .resource_mut::<p2p::PendingEvents<starboard::room::CursorUpdate>>()
            .push(p2p::Event::ZoneMessage {
                zone: identity::ZoneName::from(zone),
                from: identity::PeerId::from(from),
                payload: cursor(x),
            });
    }

    fn spectate(app: &mut App, zone: &str) {
        app.insert_resource(spectator::Spectated {
            zone: identity::ZoneName::from(zone),
            state: starboard::canvas::Canvas::default(),
            freshness: spectator::SpectateFreshness::Live,
        });
    }

    #[test]
    fn test_usage() {
        let mut app = app(joined_status(&["peer-a", "peer-b"]));
        zone_message(&mut app, "alpha", "peer-a", 1);
        zone_message(&mut app, "alpha", "peer-b", 2);
        app.update();

        let cursors = app.world().resource::<starboard::room::RemoteCursors>();
        assert_eq!(cursors.len(), 1);
        assert!(cursors.contains_key(&identity::PeerId::from("peer-b")));

        *app.world_mut().resource_mut::<DiscoveryState>() = joined_status(&["peer-a"]);
        app.update();
        assert_eq!(
            app.world()
                .resource::<starboard::room::RemoteCursors>()
                .len(),
            0
        );
    }

    #[test]
    fn a_message_from_another_zone_never_becomes_an_in_room_cursor() {
        let mut app = app(joined_status(&["peer-a", "peer-b"]));
        zone_message(&mut app, "beta", "peer-b", 8);
        app.update();

        assert_eq!(
            app.world()
                .resource::<starboard::room::RemoteCursors>()
                .len(),
            0
        );
    }

    #[test]
    fn the_spectated_zone_cursors_go_to_the_preview_not_the_room() {
        let mut app = app(joined_status(&["peer-a", "peer-b"]));
        spectate(&mut app, "beta");
        *app.world_mut().resource_mut::<DiscoveryState>() = lobby_status("beta", &["peer-b"]);
        zone_message(&mut app, "beta", "peer-b", 7);
        app.update();

        assert_eq!(
            app.world()
                .resource::<starboard::room::RemoteCursors>()
                .len(),
            0
        );
        let preview = app.world().resource::<starboard::preview::PreviewCursors>();
        assert_eq!(preview.len(), 1);
        assert_eq!(
            preview
                .get(&identity::PeerId::from("peer-b"))
                .map(|c| c.cell.x),
            Some(7)
        );
    }

    #[test]
    fn leaving_the_preview_clears_its_cursors() {
        let mut app = app(lobby_status("beta", &["peer-b"]));
        spectate(&mut app, "beta");
        zone_message(&mut app, "beta", "peer-b", 7);
        app.update();
        assert_eq!(
            app.world()
                .resource::<starboard::preview::PreviewCursors>()
                .len(),
            1
        );

        app.world_mut()
            .remove_resource::<spectator::Spectated<starboard::canvas::Canvas>>();
        app.update();
        assert_eq!(
            app.world()
                .resource::<starboard::preview::PreviewCursors>()
                .len(),
            0
        );
    }
}
