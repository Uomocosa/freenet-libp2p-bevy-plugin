use std::collections::BTreeSet;

use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{discovery, identity, p2p};

use crate::starboard;

pub fn collect_remote_cursors(
    status: Option<Res<discovery::Status>>,
    ready: Option<Res<p2p::ReadyChannel>>,
    events: Option<ResMut<p2p::Events<starboard::room::CursorUpdate>>>,
    mut cursors: ResMut<starboard::room::RemoteCursors>,
) {
    let status = status.map(Res::into_inner);
    let own = ready.and_then(|ready| ready.ready_rx.borrow().clone().map(|peer| peer.id));
    if let Some(mut events) = events {
        for event in events.take_all() {
            if let p2p::Event::RoomMessage { from, payload, .. } = event {
                if Some(&from) == own.as_ref() {
                    continue;
                }
                cursors.insert(from, payload);
            }
        }
    }
    let members: BTreeSet<&identity::PeerId> = status
        .as_ref()
        .and_then(|status| status.room.as_ref())
        .map_or_else(BTreeSet::new, |room| room.members.keys().collect());
    cursors.retain(|peer, _| members.contains(peer));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity, p2p};

    use super::collect_remote_cursors;
    use crate::starboard;

    fn joined_status(members: &[&str]) -> discovery::Status {
        let mut room_members = discovery::Members::new();
        for peer in members {
            room_members.insert(
                identity::PeerId::from(*peer),
                discovery::Member {
                    room_member: discovery::RoomMember {
                        peer: identity::Peer {
                            id: identity::PeerId::from(*peer),
                            addrs: Vec::new(),
                        },
                        updated_at: discovery::UnixTime::from_secs(0),
                        display_name: None,
                        membership: discovery::RoomMembership::InRoom,
                    },
                    status: discovery::LinkStatus::Connected,
                },
            );
        }
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(0),
                phase: discovery::RoomPhase::Ready,
                members: room_members,
            }),
            ..discovery::Status::default()
        }
    }

    fn cursor(x: u16) -> starboard::room::CursorUpdate {
        starboard::room::CursorUpdate {
            cell: starboard::canvas::CellPos { x, y: 0 },
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(joined_status(&["peer-a", "peer-b"]));
        app.init_resource::<p2p::ReadyChannel>();
        app.init_resource::<p2p::Events<starboard::room::CursorUpdate>>();
        app.init_resource::<starboard::room::RemoteCursors>();
        app.add_systems(Update, collect_remote_cursors);
        let _ = app
            .world()
            .resource::<p2p::ReadyChannel>()
            .ready_tx
            .send(Some(identity::Peer {
                id: identity::PeerId::from("peer-a"),
                addrs: Vec::new(),
            }));

        app.world_mut()
            .resource_mut::<p2p::Events<starboard::room::CursorUpdate>>()
            .push(p2p::Event::RoomMessage {
                room: identity::RoomName::from("alpha"),
                from: identity::PeerId::from("peer-a"),
                payload: cursor(1),
            });
        app.world_mut()
            .resource_mut::<p2p::Events<starboard::room::CursorUpdate>>()
            .push(p2p::Event::RoomMessage {
                room: identity::RoomName::from("alpha"),
                from: identity::PeerId::from("peer-b"),
                payload: cursor(2),
            });
        app.update();

        let cursors = app.world().resource::<starboard::room::RemoteCursors>();
        assert_eq!(cursors.len(), 1);
        assert!(cursors.contains_key(&identity::PeerId::from("peer-b")));

        *app.world_mut().resource_mut::<discovery::Status>() = joined_status(&["peer-a"]);
        app.update();
        assert_eq!(
            app.world()
                .resource::<starboard::room::RemoteCursors>()
                .len(),
            0
        );
    }
}
