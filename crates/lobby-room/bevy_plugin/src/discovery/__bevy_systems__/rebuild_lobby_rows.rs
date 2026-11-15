use bevy::prelude::*;

use crate::direct_dial;
use crate::discovery;
use crate::identity;

pub fn rebuild_lobby_rows(
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    dial: Option<Res<direct_dial::state_machine::DialState>>,
    mut rows: ResMut<discovery::LobbyRows>,
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
    *rows = discovery::LobbyRows(
        state
            .map(|state| state.into_inner().lobby_rows(&friend_ids))
            .unwrap_or_default(),
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::rebuild_lobby_rows;
    use crate::direct_dial;
    use crate::discovery;
    use crate::identity;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<discovery::LobbyRows>();
        app.add_systems(Update, rebuild_lobby_rows);
        app.update();
        let rows = app.world().resource::<discovery::LobbyRows>();
        assert_eq!(**rows, Vec::new());
    }

    #[test]
    fn rows_follow_the_lobby_and_the_friends() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("alpha"),
            discovery::Room {
                room_size: discovery::RoomSize::try_from(8).expect("8"),
                created_at: time::UnixTime::EPOCH,
                members: std::collections::BTreeMap::from([(
                    identity::PeerId::from("alice"),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: identity::PeerId::from("alice"),
                            addrs: Vec::new(),
                        },
                        display_name: None,
                        updated_at: time::UnixTime::EPOCH,
                        membership: discovery::RoomMembership::Occupant,
                    },
                )]),
            },
        );
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: Vec::new(),
        }));
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .with_lobby(lobby)
                .build(),
        );
        app.insert_resource(direct_dial::state_machine::DialState::from_peer_book(
            &book,
            time::UnixTime::from_secs(10),
        ));
        app.init_resource::<discovery::LobbyRows>();
        app.add_systems(Update, rebuild_lobby_rows);
        app.update();
        let rows = app.world().resource::<discovery::LobbyRows>();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].occupants, 1);
        assert_eq!(rows[0].friends, 1);
    }
}
