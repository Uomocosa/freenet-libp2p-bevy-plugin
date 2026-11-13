use std::sync::Arc;

use tokio::net::TcpListener;

use crate::testing;

pub async fn start_with(
    kit: Box<dyn testing::traits::FakeNetworkKit>,
) -> Result<testing::FakeNetworkServer, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| error.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    let orchestrator = testing::FakeNetworkServer(port);
    let shared = Arc::new(tokio::sync::Mutex::new(
        testing::orchestrator_shared::OrchestratorShared::from(kit),
    ));
    let accept_task = orchestrator.clone();
    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                continue;
            };
            let peer_task = accept_task.clone();
            let peer_shared = shared.clone();
            tokio::spawn(async move {
                peer_task.serve_peer(stream, peer_shared).await;
            });
        }
    });
    Ok(orchestrator)
}

#[cfg(test)]
mod tests {
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let started = testing::FakeNetworkServer::start_with(Box::new(testing::NoFakeNetworkKit))
            .await
            .expect("start");
        assert!(*started > 0);
    }
}
