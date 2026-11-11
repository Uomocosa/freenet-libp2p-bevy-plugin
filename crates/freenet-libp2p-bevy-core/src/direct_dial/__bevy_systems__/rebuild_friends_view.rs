use std::collections::BTreeSet;

use bevy::prelude::*;

use crate::direct_dial;
use crate::discovery;
use crate::identity;
use crate::peers;

pub fn rebuild_friends_view(
    dial: Option<Res<direct_dial::state_machine::DialState>>,
    discovery_state: Option<Res<discovery::state_machine::DiscoveryState>>,
    connected_peers: Query<&peers::Peer, With<peers::Connected>>,
    mut view: ResMut<direct_dial::FriendsView>,
) {
    let friend_ids: Vec<identity::PeerId> = dial
        .map(|dial| {
            dial.entries
                .iter()
                .filter(|entry| entry.kind == direct_dial::KnownPeerKind::Friend)
                .map(|entry| entry.peer.id.clone())
                .collect()
        })
        .unwrap_or_default();
    let connected: BTreeSet<identity::PeerId> = connected_peers
        .iter()
        .map(|peer| (**peer).clone())
        .collect();
    let lobby = discovery_state
        .map(|state| state.into_inner().lobby.clone())
        .unwrap_or_default();
    *view = direct_dial::FriendsView::from_friend_ids(&friend_ids, &lobby, &connected);
}
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::rebuild_friends_view;
    use crate::direct_dial;
    use crate::discovery;
    use crate::identity;
    use crate::peers;
    use crate::testing;
    use crate::time;

    fn friend_state(names: &[&str]) -> direct_dial::state_machine::DialState {
        let mut book = direct_dial::PeerBook::default();
        for name in names {
            book.add_friend(identity::Peer {
                id: identity::PeerId::from(*name),
                addrs: Vec::new(),
            });
        }
        direct_dial::state_machine::DialState::from_peer_book(&book, time::UnixTime::from_secs(10))
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(friend_state(&["alice"]));
        app.init_resource::<direct_dial::FriendsView>();
        app.add_systems(Update, rebuild_friends_view);
        app.update();
        let view = app.world().resource::<direct_dial::FriendsView>();
        assert_eq!(view.len(), 1);
        assert_eq!(view[0].name, "alice");
        assert_eq!(view[0].presence, direct_dial::FriendPresence::Offline);
    }

    #[test]
    fn presence_follows_the_connected_peers_and_the_lobby() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("room-7"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: std::collections::BTreeMap::from([(
                    identity::PeerId::from("bob"),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: identity::PeerId::from("bob"),
                            addrs: Vec::new(),
                        },
                        display_name: Some(identity::Username::from("Bob")),
                        updated_at: time::UnixTime::EPOCH,
                        membership: discovery::RoomMembership::Occupant,
                    },
                )]),
            },
        );
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(friend_state(&["alice", "bob"]));
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .with_lobby(lobby)
                .build(),
        );
        app.init_resource::<direct_dial::FriendsView>();
        app.world_mut().spawn((
            peers::Peer::from(identity::PeerId::from("bob")),
            peers::Connected,
        ));
        app.add_systems(Update, rebuild_friends_view);
        app.update();
        let view = app.world().resource::<direct_dial::FriendsView>();
        let bob = view
            .iter()
            .find(|friend| friend.id == identity::PeerId::from("bob"))
            .expect("bob");
        assert_eq!(
            bob.presence,
            direct_dial::FriendPresence::InRoom(identity::RoomName::from("room-7"))
        );
        let alice = view
            .iter()
            .find(|friend| friend.id == identity::PeerId::from("alice"))
            .expect("alice");
        assert_eq!(alice.presence, direct_dial::FriendPresence::Offline);
    }
}
