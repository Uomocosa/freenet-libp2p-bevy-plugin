use std::sync::Arc;

use atomic_delegate_macros::atomic_delegates;
use derive_more::{Deref, From};

use crate::discovery;
use crate::identity;
use crate::testing;

#[derive(Debug, Clone, Deref, From)]
#[from(forward)]
pub struct FakeNetworkServer(pub u16);

#[atomic_delegates]
impl FakeNetworkServer {
    pub async fn start() -> Result<Self, String> {}
}

#[atomic_delegates]
impl FakeNetworkServer {
    pub(crate) async fn serve_peer(
        self,
        stream: tokio::net::TcpStream,
        shared: Arc<tokio::sync::Mutex<testing::orchestrator_shared::OrchestratorShared>>,
    ) {
    }
    pub(crate) async fn send_routed_events(
        &self,
        shared: &Arc<tokio::sync::Mutex<testing::orchestrator_shared::OrchestratorShared>>,
        events: Vec<testing::RoutedEvent>,
    ) {
    }
    pub(crate) async fn send_lobby_to_game_peers(
        &self,
        shared: &Arc<tokio::sync::Mutex<testing::orchestrator_shared::OrchestratorShared>>,
        peer: &identity::PeerId,
        lobby: discovery::Lobby,
    ) {
    }
}

// no test_usage necessary
