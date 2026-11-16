use std::time::Instant;

use serde_json::Value;

use crate::testing;

pub async fn await_brp(peer: &mut testing::harness::Peer) -> Result<(), String> {
    let started = Instant::now();
    loop {
        if peer
            .client
            .call::<Value>("rpc.discover", Value::Null)
            .await
            .is_ok()
        {
            return Ok(());
        }
        if let Some(status) = peer.exit_status()? {
            return Err(peer.explain(&format!(
                "app exited before answering rpc.discover: {status}"
            )));
        }
        if started.elapsed() >= testing::harness::DISCOVER_DEADLINE {
            return Err(peer.explain("rpc.discover did not answer"));
        }
        tokio::time::sleep(testing::harness::POLL_INTERVAL).await;
    }
}

// no test_usage necessary
