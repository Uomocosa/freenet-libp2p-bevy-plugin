use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use crate::freenet;

pub async fn next_reply(client: &mut freenet::Client) -> Result<HostResponse, freenet::Error> {
    loop {
        match client.recv().await? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification { .. }) => {
                tracing::debug!(target: "freenet", "contract update notification");
            }
            other => return Ok(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use freenet_stdlib::client_api::{ContractResponse, HostResponse};
    use freenet_stdlib::prelude::{ContractCode, Parameters, WrappedContract, WrappedState};

    use super::next_reply;
    use crate::freenet;

    #[tokio::test]
    async fn test_usage() {
        let key = WrappedContract::new(
            Arc::new(ContractCode::from(vec![0; 4])),
            Parameters::from(Vec::new()),
        )
        .key;
        let (write, _write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        read_tx
            .send(Ok(HostResponse::ContractResponse(
                ContractResponse::GetResponse {
                    key,
                    contract: None,
                    state: WrappedState::new(Vec::new()),
                },
            )))
            .ok();
        drop(read_tx);
        let mut client = freenet::Client { write, read };
        let reply = next_reply(&mut client).await;
        assert!(matches!(
            reply,
            Ok(HostResponse::ContractResponse(
                ContractResponse::GetResponse { .. }
            ))
        ));
        assert!(next_reply(&mut client).await.is_err());
    }
}
