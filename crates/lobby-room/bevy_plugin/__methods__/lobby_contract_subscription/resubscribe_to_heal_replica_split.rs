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
    use super::resubscribe_to_heal_replica_split;
    use crate::testing;

    #[test]
    fn test_usage() {
        let (write, mut write_rx) = tokio::sync::mpsc::unbounded_channel();
        let (read_tx, read) = tokio::sync::mpsc::unbounded_channel();
        drop(read_tx);
        let subscription = testing::lobby_subscription(write, read);
        assert!(resubscribe_to_heal_replica_split(&subscription).is_ok());
        assert!(write_rx.try_recv().is_ok());
    }
}
