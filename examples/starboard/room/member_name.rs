use freenet_libp2p_bevy_plugin::{discovery, identity};

#[must_use]
pub fn member_name(status: &discovery::Status, peer: &identity::PeerId) -> String {
    status
        .room
        .as_ref()
        .and_then(|room| room.members.get(peer))
        .and_then(|member| member.room_member.display_name.as_ref())
        .map_or_else(|| peer.as_str().to_string(), ToString::to_string)
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::member_name;

    #[test]
    fn test_usage() {
        let peer = identity::PeerId::from("peer-a");
        let mut members = discovery::Members::new();
        members.insert(
            peer.clone(),
            discovery::Member {
                room_member: discovery::RoomMember {
                    peer: identity::Peer {
                        id: peer.clone(),
                        addrs: Vec::new(),
                    },
                    updated_at: discovery::UnixTime::from_secs(0),
                    display_name: Some(identity::Username::from("ada")),
                    presence: discovery::Presence::InRoom,
                },
                status: discovery::LinkStatus::Connected,
            },
        );
        let status = discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(0),
                phase: discovery::RoomPhase::Ready,
                members,
            }),
            ..discovery::Status::default()
        };
        assert_eq!(member_name(&status, &peer), "ada");
        let stranger = identity::PeerId::from("peer-z");
        assert_eq!(member_name(&status, &stranger), "peer-z");
    }
}
