use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};
use tokio::sync::watch::Receiver;

use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct Peer {
    pub id: identity::PeerId,
    pub addrs: Vec<identity::PeerAddr>,
}

#[atomic_delegates]
impl Peer {
    pub async fn wait_ready(ready: &mut Receiver<Option<Peer>>) -> Option<Peer> {}
}

#[cfg(test)]
mod tests {
    use super::Peer;
    use crate::identity;

    #[test]
    fn test_usage() {
        let peer = Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        };
        assert_eq!(peer.id.as_str(), "me");
    }
}
