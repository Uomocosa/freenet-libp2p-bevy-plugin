use freenet_stdlib::client_api::ClientRequest;
use tokio_tungstenite::tungstenite::Message;

use crate::freenet;

pub fn send(client: &freenet::Client, request: &ClientRequest<'_>) -> Result<(), freenet::Error> {
    let bytes = bincode::serialize(request)?;
    client
        .write
        .send(Message::Binary(bytes.into()))
        .map_err(|_| freenet::Error::ChannelSend)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::client_api::ClientRequest;

    use super::send;
    use crate::freenet;

    #[test]
    fn test_usage() {
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let client = freenet::Client { write, read };
        assert!(send(&client, &ClientRequest::Close).is_ok());
        assert!(write_rx.try_recv().is_ok());
    }
}
