use atomic_delegate_macros::atomic_delegates;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;

use crate::discovery;
use crate::identity;

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub me: identity::PeerId,
    pub params: Vec<u8>,
    pub discovery: discovery::Config,
}

#[atomic_delegates]
impl Config {
    pub async fn run_lobby(
        &self,
        subscription: discovery::freenet::LobbyContractSubscription,
        target: Receiver<Vec<discovery::state_machine::HeartbeatTarget>>,
        lobbies_tx: UnboundedSender<discovery::Lobby>,
    ) {
    }
    pub async fn poll_peers(&self, peers_tx: UnboundedSender<Vec<identity::FreenetPeer>>) {}
    pub async fn connect_retry(
        &self,
        target: &Receiver<Vec<discovery::state_machine::HeartbeatTarget>>,
    ) -> discovery::freenet::LobbyContractSubscription {
    }
}

// no test_usage necessary
