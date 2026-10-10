use freenet_stdlib::client_api::ClientRequest;
use tokio_tungstenite::tungstenite::Message;

use crate::discovery;

pub fn send(
    client: &discovery::freenet::Client,
    request: &ClientRequest<'_>,
) -> Result<(), discovery::Error> {
    let bytes = bincode::serialize(request)?;
    client
        .write
        .send(Message::Binary(bytes.into()))
        .map_err(|_| discovery::Error::ChannelSend)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::client_api::ClientRequest;

    use super::send;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let client = discovery::freenet::Client { write, read };
        assert!(send(&client, &ClientRequest::Close).is_ok());
        assert!(write_rx.try_recv().is_ok());
    }
}
