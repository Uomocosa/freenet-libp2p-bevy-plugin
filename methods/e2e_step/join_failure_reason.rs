use crate::identity;
use crate::testing;

#[must_use]
pub fn join_failure_reason(
    view: &testing::E2EStepView<'_>,
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

    fn viewed(status: &discovery::Status) -> testing::E2EStepView<'_> {
        testing::E2EStepView {
            status,
            join_error: None,
            room_message_senders: 0,
        }
    }

    #[test]
    fn test_usage() {
        let status = discovery::Status::default();
        assert_eq!(
            join_failure_reason(&viewed(&status), &identity::RoomName::from("room-1")),
            None
        );
        let failed = testing::E2EStepView {
            join_error: Some("gone"),
            ..viewed(&status)
        };
        assert_eq!(
            join_failure_reason(&failed, &identity::RoomName::from("room-1")),
            Some("gone".to_string())
        );
        let fetched = discovery::Status {
            is_lobby_fetched: true,
            ..discovery::Status::default()
        };
        assert_eq!(
            join_failure_reason(&viewed(&fetched), &identity::RoomName::from("room-1")),
            Some("room room-1 no longer exists".to_string())
        );
    }
}
