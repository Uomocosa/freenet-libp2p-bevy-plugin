use std::collections::BTreeSet;

use bevy::prelude::Resource;
use derive_more::Deref;

use crate::direct_dial;
use crate::identity;
use crate::presence;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref)]
pub struct FriendsView(pub Vec<direct_dial::FriendView>);

impl FriendsView {
    #[must_use]
    pub fn from_friend_ids(
        friend_ids: &[identity::PeerId],
        presence: &presence::PeerPresence,
        connected: &BTreeSet<identity::PeerId>,
    ) -> Self {
        let mut entries: Vec<direct_dial::FriendView> = friend_ids
            .iter()
            .map(|id| {
                let place = presence.get(id).and_then(|whereabouts| {
                    whereabouts
                        .place
                        .as_ref()
                        .map(|room| (room, whereabouts.display_name.as_ref()))
                });
                let (friend_presence, known_name) = match place {
                    Some((room, display_name)) => {
                        let name = display_name
                            .filter(|username| !username.is_empty())
                            .map(ToString::to_string);
                        (direct_dial::FriendPresence::InRoom(room.clone()), name)
                    }
                    None if connected.contains(id) => (direct_dial::FriendPresence::Online, None),
                    None => (direct_dial::FriendPresence::Offline, None),
                };
                let name =
                    known_name.unwrap_or_else(|| identity::ShortPeerId::from(id).to_string());
                direct_dial::FriendView {
                    id: id.clone(),
                    name,
                    presence: friend_presence,
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
    use crate::identity;
    use crate::presence;

    const LONG_ID: &str = "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC";

    fn bob_in_room_7() -> presence::PeerPresence {
        presence::PeerPresence(BTreeMap::from([(
            identity::PeerId::from("bob"),
            presence::Whereabouts {
                place: Some(identity::RoomName::from("room-7")),
                display_name: Some(identity::Username::from("Bob")),
            },
        )]))
    }

    #[test]
    fn test_usage() {
        let view =
            FriendsView::from_friend_ids(&[], &presence::PeerPresence::default(), &BTreeSet::new());
        assert!(view.is_empty());
    }

    #[test]
    fn offline_connected_and_in_room_presence() {
        let connected = BTreeSet::from([identity::PeerId::from(LONG_ID)]);
        let view = FriendsView::from_friend_ids(
            &[
                identity::PeerId::from("alice"),
                identity::PeerId::from(LONG_ID),
                identity::PeerId::from("bob"),
            ],
            &bob_in_room_7(),
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
        assert_eq!(online.presence, direct_dial::FriendPresence::Online);
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
            &presence::PeerPresence::default(),
            &connected,
        );
        let names: Vec<&str> = view.iter().map(|friend| friend.name.as_str()).collect();
        assert_eq!(names, vec!["mia", "zoe", "adam"]);
    }

    #[test]
    fn a_friend_without_a_place_is_not_in_the_room() {
        let placeless = presence::PeerPresence(BTreeMap::from([(
            identity::PeerId::from("bob"),
            presence::Whereabouts {
                place: None,
                display_name: Some(identity::Username::from("Bob")),
            },
        )]));
        let connected = BTreeSet::from([identity::PeerId::from("bob")]);
        let view =
            FriendsView::from_friend_ids(&[identity::PeerId::from("bob")], &placeless, &connected);
        assert_eq!(view[0].presence, direct_dial::FriendPresence::Online);
        assert_eq!(view[0].name, "bob");
    }
}
