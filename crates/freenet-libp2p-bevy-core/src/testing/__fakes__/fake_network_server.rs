use std::sync::Arc;

use atomic_delegate_macros::atomic_delegates;
use derive_more::{Deref, From};

use crate::identity;
use crate::testing;

#[derive(Debug, Clone, Deref, From)]
#[from(forward)]
pub struct FakeNetworkServer(pub u16);

impl FakeNetworkServer {
    #[rustfmt::skip]
    pub async fn start() -> Result<Self, String> { Self::start_with(Box::new(testing::NoFakeNetworkKit)).await }
}

#[atomic_delegates]
impl FakeNetworkServer {
    pub async fn start_with(kit: Box<dyn testing::traits::FakeNetworkKit>) -> Result<Self, String> {
    }
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
    pub(crate) async fn send_fan_out(
        &self,
        shared: &Arc<tokio::sync::Mutex<testing::orchestrator_shared::OrchestratorShared>>,
        game: &identity::GameId,
        fan_outs: Vec<testing::FanOut>,
    ) {
    }
}

// no test_usage necessary

#[cfg(test)]
mod tests {
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let started = testing::FakeNetworkServer::start().await.expect("start");
        assert!(*started > 0);
    }
}
