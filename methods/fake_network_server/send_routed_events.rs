use std::sync::Arc;

use crate::testing;

pub async fn send_routed_events(
    task: &testing::FakeNetworkServer,
    shared: &Arc<tokio::sync::Mutex<testing::orchestrator_shared::OrchestratorShared>>,
    events: Vec<testing::RoutedEvent>,
) {
    let _ = *task;
    for event in events {
        let senders = {
            let guard = shared.lock().await;
            guard.senders.get(&event.to).cloned().unwrap_or_default()
        };
        for sender in senders {
            sender
                .send(testing::FakeWireServerFrame::Libp2pEvent(
                    event.event.clone(),
                ))
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_usage() {
        let _ = stringify!(send_routed_events);
    }
}
