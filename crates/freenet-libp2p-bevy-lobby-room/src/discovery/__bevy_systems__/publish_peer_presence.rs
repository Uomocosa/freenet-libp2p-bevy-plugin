use bevy::prelude::*;

use crate::discovery;
use crate::presence;

pub fn publish_peer_presence(
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    mut presence: ResMut<presence::PeerPresence>,
) {
    if state.as_ref().is_some_and(|state| !state.is_changed()) {
        return;
    }
    let mut next = presence::PeerPresence::default();
    if let Some(state) = state {
        for (room_name, room) in state.lobby.iter() {
            for (peer_id, member) in &room.members {
                if member.membership != discovery::RoomMembership::Occupant {
                    continue;
                }
                next.entry(peer_id.clone())
                    .or_insert_with(|| presence::Whereabouts {
                        place: Some(room_name.clone()),
                        display_name: member.display_name.clone(),
                    });
            }
        }
    }
    presence.set_if_neq(next);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bevy::prelude::*;

    use super::publish_peer_presence;
    use crate::discovery;
    use crate::identity;
    use crate::presence;
    use crate::testing;
    use crate::time;

    fn room_with(members: &[(&str, &str, discovery::RoomMembership)]) -> discovery::Room {
        discovery::Room {
            room_size: discovery::RoomSize::DEFAULT,
            created_at: time::UnixTime::EPOCH,
            members: members
                .iter()
                .map(|(id, name, membership)| {
                    (
                        identity::PeerId::from(*id),
                        discovery::RoomMember {
                            peer: identity::Peer {
                                id: identity::PeerId::from(*id),
                                addrs: Vec::new(),
                            },
                            display_name: Some(identity::Username::from(*name)),
                            updated_at: time::UnixTime::EPOCH,
                            membership: *membership,
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }

    fn app_with_lobby(lobby: discovery::Lobby) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .with_lobby(lobby)
                .build(),
        );
        app.init_resource::<presence::PeerPresence>();
        app.add_systems(Update, publish_peer_presence);
        app
    }

    fn presence_of(app: &App) -> &presence::PeerPresence {
        app.world().resource::<presence::PeerPresence>()
    }

    #[test]
    fn test_usage() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("room-7"),
            room_with(&[("bob", "Bob", discovery::RoomMembership::Occupant)]),
        );
        let mut app = app_with_lobby(lobby);
        app.update();
        assert_eq!(
            presence_of(&app).get(&identity::PeerId::from("bob")),
            Some(&presence::Whereabouts {
                place: Some(identity::RoomName::from("room-7")),
                display_name: Some(identity::Username::from("Bob")),
            })
        );
    }

    #[test]
    fn a_history_holder_publishes_no_place() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("room-7"),
            room_with(&[("bob", "Bob", discovery::RoomMembership::HistoryHolder)]),
        );
        let mut app = app_with_lobby(lobby);
        app.update();
        assert!(presence_of(&app).is_empty());
    }

    #[test]
    fn a_peer_in_two_rooms_takes_the_first_room_by_name() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("room-b"),
            room_with(&[("bob", "Bob", discovery::RoomMembership::Occupant)]),
        );
        lobby.insert(
            identity::RoomName::from("room-a"),
            room_with(&[("bob", "Bob", discovery::RoomMembership::Occupant)]),
        );
        let mut app = app_with_lobby(lobby);
        app.update();
        let place = presence_of(&app)
            .get(&identity::PeerId::from("bob"))
            .and_then(|whereabouts| whereabouts.place.clone());
        assert_eq!(place, Some(identity::RoomName::from("room-a")));
    }

    #[test]
    fn an_unchanged_state_does_not_rebuild_the_presence() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("room-7"),
            room_with(&[("bob", "Bob", discovery::RoomMembership::Occupant)]),
        );
        let mut app = app_with_lobby(lobby);
        app.update();
        app.world_mut()
            .resource_mut::<presence::PeerPresence>()
            .clear();
        app.update();
        assert!(presence_of(&app).is_empty());
    }
}
