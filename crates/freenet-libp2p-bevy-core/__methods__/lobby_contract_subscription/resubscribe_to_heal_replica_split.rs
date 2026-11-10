use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::StateSummary;

use crate::discovery;

pub fn resubscribe_to_heal_replica_split(
    state: &discovery::LobbyContractSubscription,
) -> Result<(), discovery::Error> {
    let summary = bincode::serialize(&state.lobby)?;
    let request = ContractRequest::Subscribe {
        key: *state.key.id(),
        summary: Some(StateSummary::from(summary)),
    };
    state.client.send(&ClientRequest::ContractOp(request))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use freenet_stdlib::prelude::{ContractCode, ContractKey, Parameters, WrappedContract};

    use super::resubscribe_to_heal_replica_split;
    use crate::{discovery, freenet};

    fn contract_key() -> ContractKey {
        WrappedContract::new(
            Arc::new(ContractCode::from(discovery::LOBBY_CONTRACT_WASM.to_vec())),
            Parameters::from(Vec::new()),
        )
        .key
    }

    #[test]
    fn test_usage() {
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        let subscription = discovery::LobbyContractSubscription {
            client: freenet::Client { write, read },
            key: contract_key(),
            lobby: discovery::Lobby::default(),
        };
        assert!(resubscribe_to_heal_replica_split(&subscription).is_ok());
        assert!(write_rx.try_recv().is_ok());
    }
}
