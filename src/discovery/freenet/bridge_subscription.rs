use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::StateSummary;

use crate::discovery;

pub fn bridge_subscription(
    subscription: &mut discovery::freenet::LobbyContractSubscription,
) -> Result<(), discovery::Error> {
    let summary = bincode::serialize(&subscription.lobby)?;
    let request = ContractRequest::Subscribe {
        key: *subscription.key.id(),
        summary: Some(StateSummary::from(summary)),
    };
    subscription
        .client
        .send(&ClientRequest::ContractOp(request))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use freenet_stdlib::prelude::{ContractCode, ContractKey, Parameters, WrappedContract};

    use super::bridge_subscription;
    use crate::discovery;

    fn contract_key() -> ContractKey {
        WrappedContract::new(
            Arc::new(ContractCode::from(
                discovery::freenet::contract_wasm().to_vec(),
            )),
            Parameters::from(Vec::new()),
        )
        .key
    }

    #[test]
    fn test_usage() {
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let mut subscription = discovery::freenet::LobbyContractSubscription {
            client: discovery::freenet::Client { write, read },
            key: contract_key(),
            lobby: discovery::Lobby::default(),
        };
        assert!(bridge_subscription(&mut subscription).is_ok());
        assert!(write_rx.try_recv().is_ok());
    }
}
