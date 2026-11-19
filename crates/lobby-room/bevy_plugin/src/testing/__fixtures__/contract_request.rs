use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use tokio_tungstenite::tungstenite::Message;

#[must_use]
pub fn contract_request(message: &Message) -> Option<ContractRequest<'_>> {
    let Message::Binary(bytes) = message else {
        return None;
    };
    let request = bincode::deserialize::<ClientRequest<'_>>(bytes.as_ref()).ok()?;
    let ClientRequest::ContractOp(request) = request else {
        return None;
    };
    Some(request)
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
    use tokio_tungstenite::tungstenite::Message;

    use crate::testing;

    use super::contract_request;

    #[test]
    fn test_usage() {
        let key = testing::contract_key();
        let request = ClientRequest::ContractOp(ContractRequest::Get {
            key: *key.id(),
            return_contract_code: false,
            subscribe: false,
            blocking_subscribe: false,
        });
        let bytes = bincode::serialize(&request).expect("request serialized");
        let message = Message::Binary(bytes.into());
        assert!(matches!(
            contract_request(&message),
            Some(ContractRequest::Get { .. })
        ));
        assert!(contract_request(&Message::Ping(Vec::new().into())).is_none());
    }
}
