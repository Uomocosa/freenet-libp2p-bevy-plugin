use crate::freenet;

#[must_use]
pub fn disconnected_client() -> freenet::Client {
    let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
    let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
    drop(read_tx);
    freenet::Client { write, read }
}

#[cfg(test)]
mod tests {
    use super::disconnected_client;

    #[tokio::test]
    async fn test_usage() {
        assert!(disconnected_client().recv().await.is_err());
    }
}
