use freenet_stdlib::client_api::HostResponse;

use crate::freenet;

#[must_use]
pub fn reply_client(reply: HostResponse) -> freenet::Client {
    let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
    let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
    read_tx.send(Ok(reply)).ok();
    freenet::Client { write, read }
}

#[cfg(test)]
mod tests {
    use crate::testing;

    use super::reply_client;

    #[tokio::test]
    async fn test_usage() {
        let reply = testing::contract_put_response(testing::contract_key());
        assert!(reply_client(reply).recv().await.is_ok());
    }
}
