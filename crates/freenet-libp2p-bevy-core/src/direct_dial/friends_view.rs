use std::collections::BTreeSet;

use bevy::prelude::Resource;
use derive_more::Deref;

use crate::direct_dial;
use crate::discovery;
use crate::identity;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref)]
pub struct FriendsView(pub Vec<direct_dial::FriendView>);

impl FriendsView {
    #[must_use]
    pub fn from_friend_ids(
        friend_ids: &[identity::PeerId],
        lobby: &discovery::Lobby,
        connected: &BTreeSet<identity::PeerId>,
    ) -> Self {
        let mut entries: Vec<direct_dial::FriendView> = friend_ids
            .iter()
            .map(|id| {
                let in_room = lobby.iter().find_map(|(room_name, room)| {
                    let member = room.members.get(id)?;
                    (member.membership == discovery::RoomMembership::Occupant)
                        .then_some((room_name, member))
                });
                let (presence, known_name) = match in_room {
                    Some((room_name, member)) => {
                        let name = member
                            .display_name
                            .as_ref()
                            .filter(|username| !username.is_empty())
                            .map(ToString::to_string);
                        (direct_dial::FriendPresence::InRoom(room_name.clone()), name)
                    }
                    None if connected.contains(id) => (direct_dial::FriendPresence::InLobby, None),
                    None => (direct_dial::FriendPresence::Offline, None),
                };
                let name =
                    known_name.unwrap_or_else(|| identity::ShortPeerId::from(id).to_string());
                direct_dial::FriendView {
                    id: id.clone(),
                    name,
                    presence,
                }
            })
            .collect();
        entries.sort_by(|a, b| {
            let a_is_offline = matches!(a.presence, direct_dial::FriendPresence::Offline);
            let b_is_offline = matches!(b.presence, direct_dial::FriendPresence::Offline);
            a_is_offline
                .cmp(&b_is_offline)
                .then_with(|| a.name.cmp(&b.name))
        });
        Self(entries)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use super::FriendsView;
    use crate::direct_dial;
    use crate::discovery;
    use crate::identity;
    use crate::time;

    const LONG_ID: &str = "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC";

    fn room_with(members: &[(&str, Option<&str>, discovery::RoomMembership)]) -> discovery::Room {
        discovery::Room {
            room_size: discovery::RoomSize::DEFAULT,
            created_at: time::UnixTime::EPOCH,
            members: members
                .iter()
                .map(|(id, display_name, membership)| {
                    (
                        identity::PeerId::from(*id),
                        discovery::RoomMember {
                            peer: identity::Peer {
                                id: identity::PeerId::from(*id),
                                addrs: Vec::new(),
                            },
                            display_name: display_name.map(identity::Username::from),
                            updated_at: time::UnixTime::EPOCH,
                            membership: *membership,
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn test_usage() {
        let view =
            FriendsView::from_friend_ids(&[], &discovery::Lobby::default(), &BTreeSet::new());
        assert!(view.is_empty());
    }

    #[test]
    fn offline_connected_and_in_room_presence() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("room-7"),
            room_with(&[("bob", Some("Bob"), discovery::RoomMembership::Occupant)]),
        );
        let connected = BTreeSet::from([identity::PeerId::from(LONG_ID)]);
        let view = FriendsView::from_friend_ids(
            &[
                identity::PeerId::from("alice"),
                identity::PeerId::from(LONG_ID),
                identity::PeerId::from("bob"),
            ],
            &lobby,
            &connected,
        );
        assert_eq!(view.len(), 3);
        let bob = view
            .iter()
            .find(|friend| friend.id == identity::PeerId::from("bob"))
            .expect("bob");
        assert_eq!(
            bob.presence,
            direct_dial::FriendPresence::InRoom(identity::RoomName::from("room-7"))
        );
        assert_eq!(bob.name, "Bob");
        let online = view
            .iter()
            .find(|friend| friend.id == identity::PeerId::from(LONG_ID))
            .expect("online friend");
        assert_eq!(online.presence, direct_dial::FriendPresence::InLobby);
        assert_eq!(online.name, "12D3Koo...99RC");
        let offline = view
            .iter()
            .find(|friend| friend.id == identity::PeerId::from("alice"))
            .expect("offline friend");
        assert_eq!(offline.presence, direct_dial::FriendPresence::Offline);
        assert_eq!(offline.name, "alice");
    }

    #[test]
    fn online_first_then_by_name() {
        let connected =
            BTreeSet::from([identity::PeerId::from("zoe"), identity::PeerId::from("mia")]);
        let view = FriendsView::from_friend_ids(
            &[
                identity::PeerId::from("zoe"),
                identity::PeerId::from("adam"),
                identity::PeerId::from("mia"),
            ],
            &discovery::Lobby::default(),
            &connected,
        );
        let names: Vec<&str> = view.iter().map(|friend| friend.name.as_str()).collect();
        assert_eq!(names, vec!["mia", "zoe", "adam"]);
    }

    #[test]
    fn a_history_holder_friend_is_not_in_the_room() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("room-7"),
            room_with(&[("bob", None, discovery::RoomMembership::HistoryHolder)]),
        );
        let connected = BTreeSet::from([identity::PeerId::from("bob")]);
        let view =
            FriendsView::from_friend_ids(&[identity::PeerId::from("bob")], &lobby, &connected);
        assert_eq!(view[0].presence, direct_dial::FriendPresence::InLobby);
    }
}
