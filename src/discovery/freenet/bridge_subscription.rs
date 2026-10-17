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
    #[test]
    fn test_usage() {
        std::hint::black_box(super::bridge_subscription);
    }
}
