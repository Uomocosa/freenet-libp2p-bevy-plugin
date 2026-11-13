use crate::identity;
use crate::testing;

#[must_use]
pub fn join_failure_reason(
    view: &testing::LobbyStepView<'_>,
    name: &identity::RoomName,
) -> Option<String> {
    if let Some(reason) = view.join_error {
        return Some(reason.to_string());
    }
    let status = view.status;
    if status.room.is_some() || !status.is_lobby_fetched || status.lobby.contains_key(name) {
        None
    } else {
        Some(format!("room {name} no longer exists"))
    }
}

#[cfg(test)]
mod tests {
    use super::join_failure_reason;
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    fn viewed(status: &discovery::state_machine::DiscoveryState) -> testing::LobbyStepView<'_> {
        testing::LobbyStepView {
            status,
            join_error: None,
        }
    }

    #[test]
    fn test_usage() {
        let status = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        assert_eq!(
            join_failure_reason(&viewed(&status), &identity::RoomName::from("room-1")),
            None
        );
        let failed = testing::LobbyStepView {
            join_error: Some("gone"),
            ..viewed(&status)
        };
        assert_eq!(
            join_failure_reason(&failed, &identity::RoomName::from("room-1")),
            Some("gone".to_string())
        );
        let fetched = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .lobby_fetched()
            .build();
        assert_eq!(
            join_failure_reason(&viewed(&fetched), &identity::RoomName::from("room-1")),
            Some("room room-1 no longer exists".to_string())
        );
    }
}
