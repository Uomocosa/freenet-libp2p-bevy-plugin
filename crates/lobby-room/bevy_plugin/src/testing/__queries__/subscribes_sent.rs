use freenet_stdlib::client_api::ClientRequest;
use freenet_stdlib::client_api::ContractRequest;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_tungstenite::tungstenite::Message;

#[must_use]
pub fn subscribes_sent(written: &mut UnboundedReceiver<Message>) -> u32 {
    let mut subscribes: u32 = 0;
    while let Ok(Message::Binary(bytes)) = written.try_recv() {
        if let Ok(ClientRequest::ContractOp(ContractRequest::Subscribe { .. })) =
            bincode::deserialize::<ClientRequest<'_>>(&bytes)
        {
            subscribes = subscribes.saturating_add(1);
        }
    }
    subscribes
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::client_api::ClientRequest;
    use freenet_stdlib::client_api::ContractRequest;
    use tokio_tungstenite::tungstenite::Message;

    use crate::testing;

    use super::subscribes_sent;

    #[test]
    fn test_usage() {
        let (write, mut written) = tokio::sync::mpsc::unbounded_channel();
        let request = ClientRequest::ContractOp(ContractRequest::Subscribe {
            key: *testing::contract_key().id(),
            summary: None,
        });
        write
            .send(Message::Binary(
                bincode::serialize(&request).unwrap_or_default().into(),
            ))
            .ok();
        assert_eq!(subscribes_sent(&mut written), 1);
        assert_eq!(subscribes_sent(&mut written), 0);
    }
}
