use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;

use crate::discovery;
use crate::identity;
use crate::time;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbyLeaseLimits {
    pub lease_ttl: Duration,
    pub remote_clock_skew_margin: Duration,
}

#[atomic_delegates]
impl LobbyLeaseLimits {
    pub fn drop_members_whose_lease_lapsed(
        &self,
        lobby: &mut discovery::Lobby,
        observed: &std::collections::BTreeMap<
            (identity::RoomName, identity::PeerId),
            discovery::state_machine::ObservedLobbyHeartbeat,
        >,
        now: time::UnixTime,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::LobbyLeaseLimits;

    #[test]
    fn test_usage() {
        let limits = LobbyLeaseLimits {
            lease_ttl: Duration::from_secs(90),
            remote_clock_skew_margin: Duration::from_secs(600),
        };
        assert_eq!(limits.lease_ttl, Duration::from_secs(90));
    }
}
