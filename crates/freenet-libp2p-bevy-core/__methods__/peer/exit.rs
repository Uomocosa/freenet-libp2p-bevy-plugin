use std::time::Instant;

use serde_json::Value;

use crate::testing;

pub async fn exit(peer: &mut testing::harness::Peer) -> Result<(), String> {
    let _ = peer.client.call::<Value>("e2e/exit", Value::Null).await;
    let started = Instant::now();
    loop {
        if let Some(status) = peer.exit_status()? {
            return if status.success() {
                Ok(())
            } else {
                Err(peer.explain(&format!("app exited with {status}")))
            };
        }
        if started.elapsed() >= testing::harness::EXIT_DEADLINE {
            return Err(peer.explain("app did not exit after e2e/exit"));
        }
        tokio::time::sleep(testing::harness::POLL_INTERVAL).await;
    }
}

// no test_usage necessary
