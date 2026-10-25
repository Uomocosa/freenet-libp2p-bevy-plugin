use std::time::{Duration, Instant};

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
    pub(crate) fn initial_lobby(
        me: &identity::PeerId,
        room_size: discovery::RoomSize,
        target: Option<&discovery::state_machine::HeartbeatTarget>,
    ) -> discovery::Lobby {
    }
}

#[rustfmt::skip]
impl Config {
    pub(crate) fn is_due(last: Option<Instant>, now: Instant, every: Duration) -> bool {
        last.is_none_or(|last| now.duration_since(last) >= every)
    }
}

// no test_usage necessary
