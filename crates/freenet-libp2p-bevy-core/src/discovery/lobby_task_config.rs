use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;
use tokio::time::Instant;

use crate::discovery;
use crate::identity;

#[derive(Clone)]
pub struct LobbyTaskConfig {
    pub host: String,
    pub port: u16,
    pub me: identity::PeerId,
    pub params: Vec<u8>,
    pub discovery: discovery::Config,
}

#[atomic_delegates]
impl LobbyTaskConfig {
    pub async fn run_lobby_sync_loop(
        &self,
        subscription: discovery::LobbyContractSubscription,
        target: Receiver<Vec<discovery::state_machine::HeartbeatTarget>>,
        lobbies_tx: UnboundedSender<discovery::Lobby>,
    ) {
    }
    pub async fn poll_peers(&self, peers_tx: UnboundedSender<Vec<identity::FreenetPeer>>) {}
    pub async fn connect_retry(
        &self,
        target: &Receiver<Vec<discovery::state_machine::HeartbeatTarget>>,
    ) -> discovery::LobbyContractSubscription {
    }
}

#[atomic_delegates]
impl LobbyTaskConfig {
    pub(crate) async fn spawn_lobby_tasks(&self, channels: discovery::LobbyTaskChannels) {}
    pub(crate) fn seed_lobby_for_deploy(
        me: &identity::PeerId,
        room_size: discovery::RoomSize,
        target: Option<&discovery::state_machine::HeartbeatTarget>,
    ) -> discovery::Lobby {
    }
}

#[rustfmt::skip]
impl LobbyTaskConfig {
    pub(crate) fn is_due(last: Option<Instant>, now: Instant, every: Duration) -> bool {
        last.is_none_or(|last| now.duration_since(last) >= every)
    }
}

// no test_usage necessary
