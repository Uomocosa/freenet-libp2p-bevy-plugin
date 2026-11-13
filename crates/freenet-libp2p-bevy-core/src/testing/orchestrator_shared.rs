use std::collections::HashMap;

use atomic_delegate_macros::atomic_delegates;
use tokio::sync::mpsc::UnboundedSender;

use crate::identity;
use crate::testing;
use crate::time;

pub struct OrchestratorShared {
    pub routers: HashMap<identity::GameId, testing::PeerMessageRouter>,
    pub senders: HashMap<identity::PeerId, Vec<UnboundedSender<testing::FakeWireServerFrame>>>,
    pub peer_games: HashMap<identity::PeerId, identity::GameId>,
    pub(crate) kit: Box<dyn testing::traits::FakeNetworkKit>,
}

impl OrchestratorShared {}

impl Default for OrchestratorShared {
    fn default() -> Self {
        let kit: Box<dyn testing::traits::FakeNetworkKit> = Box::new(testing::NoFakeNetworkKit);
        Self::from(kit)
    }
}

impl From<Box<dyn testing::traits::FakeNetworkKit>> for OrchestratorShared {
    fn from(kit: Box<dyn testing::traits::FakeNetworkKit>) -> Self {
        Self {
            routers: HashMap::new(),
            senders: HashMap::new(),
            peer_games: HashMap::new(),
            kit,
        }
    }
}

#[atomic_delegates]
impl OrchestratorShared {
    pub fn router_for(&mut self, game: &identity::GameId) -> &mut testing::PeerMessageRouter {}
}

impl OrchestratorShared {
    pub fn add_sender(
        &mut self,
        peer: &identity::PeerId,
        game: &identity::GameId,
        sender: UnboundedSender<testing::FakeWireServerFrame>,
    ) {
        self.peer_games.insert(peer.clone(), game.clone());
        self.senders.entry(peer.clone()).or_default().push(sender);
    }
}

#[atomic_delegates]
impl OrchestratorShared {
    pub(crate) fn handle_client_frame(
        &mut self,
        peer: &identity::PeerId,
        frame: testing::FakeWireClientFrame,
    ) -> Vec<testing::RoutedEvent> {
    }
    pub(crate) fn route_extension(
        &mut self,
        peer: &identity::PeerId,
        bytes: Vec<u8>,
        now: time::UnixTime,
    ) -> Vec<testing::FanOut> {
    }
    pub(crate) fn remove_connection(
        &mut self,
        peer: &identity::PeerId,
        now: time::UnixTime,
    ) -> (Vec<testing::RoutedEvent>, Vec<testing::FanOut>) {
    }
    pub(crate) fn connect_peer(
        &mut self,
        peer: &identity::Peer,
        game: &identity::GameId,
        sender: tokio::sync::mpsc::UnboundedSender<testing::FakeWireServerFrame>,
    ) {
    }
}
#[cfg(test)]
mod tests {
    use super::OrchestratorShared;
    use crate::identity;

    #[test]
    fn test_usage() {
        let mut shared = OrchestratorShared::default();
        let game = identity::GameId::from("game");
        shared.router_for(&game);
        assert!(shared.routers.contains_key(&game));
    }
}
