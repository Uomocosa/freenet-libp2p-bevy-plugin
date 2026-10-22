use freenet_libp2p_bevy_plugin::discovery;

#[must_use]
pub fn is_in_room(status: &discovery::Status) -> bool {
    status
        .room
        .as_ref()
        .is_some_and(|room| room.phase == discovery::JoinPhase::Ready)
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::is_in_room;

    fn status_with(phase: discovery::JoinPhase) -> discovery::Status {
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(0),
                phase,
                members: discovery::Members::new(),
            }),
            ..discovery::Status::default()
        }
    }

    #[test]
    fn test_usage() {
        assert!(!is_in_room(&discovery::Status::default()));
        assert!(!is_in_room(&status_with(discovery::JoinPhase::Joining {
            since: discovery::UnixTime::from_secs(0)
        })));
        assert!(is_in_room(&status_with(discovery::JoinPhase::Ready)));
    }
}
