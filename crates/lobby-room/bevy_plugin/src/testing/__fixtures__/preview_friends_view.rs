use freenet_libp2p_bevy_core::direct_dial;
use freenet_libp2p_bevy_core::identity;

#[must_use]
pub fn preview_friends_view() -> direct_dial::FriendsView {
    direct_dial::FriendsView(vec![
        direct_dial::FriendView {
            id: identity::PeerId::from("bob"),
            name: String::from("bob"),
            presence: direct_dial::FriendPresence::InZone(identity::ZoneName::from("room-7")),
        },
        direct_dial::FriendView {
            id: identity::PeerId::from("mia"),
            name: String::from("mia"),
            presence: direct_dial::FriendPresence::Online,
        },
        direct_dial::FriendView {
            id: identity::PeerId::from("adam"),
            name: String::from("12D3Koo...99RC"),
            presence: direct_dial::FriendPresence::Offline,
        },
    ])
}

#[cfg(test)]
mod tests {
    use super::preview_friends_view;

    #[test]
    fn test_usage() {
        let view = preview_friends_view();
        assert_eq!(view.len(), 3);
    }
}
