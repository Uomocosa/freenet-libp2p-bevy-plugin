use crate::direct_dial;
use crate::testing;
use crate::time;

#[must_use]
pub fn dial_state_with_friend(now: time::UnixTime) -> direct_dial::state_machine::DialState {
    let mut book = direct_dial::PeerBook::default();
    book.add_friend(testing::peer("alice", &["/ip4/127.0.0.1/tcp/4001"]));
    direct_dial::state_machine::DialState::from_peer_book(&book, now)
}

#[cfg(test)]
mod tests {
    use crate::time;

    use super::dial_state_with_friend;

    #[test]
    fn test_usage() {
        let state = dial_state_with_friend(time::UnixTime::from_secs(10));
        assert_eq!(state.entries.len(), 1);
    }
}
