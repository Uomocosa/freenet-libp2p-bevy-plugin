use crate::discovery;
use discovery::freenet::basic::constants;

pub async fn connect_socket(host: &str, port: u16) -> discovery::freenet::Client {
    loop {
        match discovery::freenet::Client::connect(host, port).await {
            Ok(client) => return client,
            Err(e) => {
                tracing::warn!(
                    target: "discovery",
                    error = %e,
                    "freenet websocket connect failed, retrying"
                );
                tokio::time::sleep(constants::CONNECT_RETRY).await;
            }
        }
    }
}

// no test_usage necessary
