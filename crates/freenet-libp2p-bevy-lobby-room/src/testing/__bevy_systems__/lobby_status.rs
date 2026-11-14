use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use crate::discovery;
use crate::testing;
use crate::ui;

pub fn lobby_status(
    In(_params): In<Option<Value>>,
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    join_error: Option<Res<ui::discovery::JoinError>>,
) -> BrpResult {
    let mut status = state.map_or_else(testing::LobbyStatus::default, |state| {
        testing::LobbyStatus {
            rooms: state
                .lobby
                .keys()
                .map(|room| room.as_str().to_owned())
                .collect(),
            joined_room: state
                .room
                .as_ref()
                .map(|room| room.name.as_str().to_owned()),
            joined_ready: state
                .room
                .as_ref()
                .is_some_and(|room| room.phase == discovery::JoinPhase::Ready),
            other_members: state.room.as_ref().map_or(0, |room| room.members.len()),
            connected: state.connected.len(),
            lobby_fetched: state.is_lobby_fetched,
            join_error: None,
        }
    });
    status.join_error = join_error.and_then(|error| (**error).clone());
    serde_json::to_value(status).map_err(BrpError::internal)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::discovery;
    use crate::identity;
    use crate::testing;
    use crate::time;
    use crate::ui;

    fn member(peer: &str) -> discovery::RoomMember {
        discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from(peer),
                addrs: Vec::new(),
            },
            display_name: None,
            updated_at: time::UnixTime::from_secs(1),
            membership: discovery::RoomMembership::Occupant,
        }
    }

    fn room_with(members: &[&str]) -> discovery::Room {
        discovery::Room {
            room_size: discovery::RoomSize::DEFAULT,
            created_at: time::UnixTime::from_secs(1),
            members: members
                .iter()
                .map(|peer| (identity::PeerId::from(*peer), member(peer)))
                .collect(),
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let system = app.world_mut().register_system(super::lobby_status);
        let before = app.world_mut().run_system_with(system, None);
        let empty: Option<testing::LobbyStatus> = before
            .ok()
            .and_then(Result::ok)
            .and_then(|value| serde_json::from_value(value).ok());
        assert_eq!(empty, Some(testing::LobbyStatus::default()));

        let mut lobby = discovery::Lobby::default();
        lobby.insert(identity::RoomName::from("beta"), room_with(&["peer-b"]));
        lobby.insert(
            identity::RoomName::from("alpha"),
            room_with(&["peer-a", "me"]),
        );
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .lobby_fetched()
            .with_lobby(lobby)
            .in_room(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: time::UnixTime::from_secs(1),
                phase: discovery::JoinPhase::Ready,
                members: room_with(&["peer-a", "me"]).members,
                how_entered: discovery::HowRoomWasEntered::Created,
            })
            .with_connected(vec![identity::PeerId::from("peer-a")])
            .build();
        app.insert_resource(state);
        let result = app.world_mut().run_system_with(system, None);
        let status: Option<testing::LobbyStatus> = result
            .ok()
            .and_then(Result::ok)
            .and_then(|value| serde_json::from_value(value).ok());
        assert_eq!(
            status,
            Some(testing::LobbyStatus {
                rooms: vec!["alpha".to_string(), "beta".to_string()],
                joined_room: Some("alpha".to_string()),
                joined_ready: true,
                other_members: 2,
                connected: 1,
                lobby_fetched: true,
                join_error: None,
            })
        );
    }

    #[test]
    fn join_error_is_read_from_the_join_error_resource() {
        let mut app = App::new();
        let system = app.world_mut().register_system(super::lobby_status);
        app.insert_resource(ui::discovery::JoinError(Some(String::from(
            "Room alpha no longer exists (all players left)",
        ))));
        let result = app.world_mut().run_system_with(system, None);
        let status: Option<testing::LobbyStatus> = result
            .ok()
            .and_then(Result::ok)
            .and_then(|value| serde_json::from_value(value).ok());
        assert_eq!(
            status.and_then(|status| status.join_error),
            Some(String::from(
                "Room alpha no longer exists (all players left)"
            ))
        );
    }
}
