use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::StateSummary;

use crate::discovery;

pub fn bridge_subscription(
    lobby_client: &mut discovery::freenet::LobbyClient,
) -> Result<(), discovery::Error> {
    let summary = bincode::serialize(&lobby_client.lobby)?;
    let request = ContractRequest::Subscribe {
        key: *lobby_client.key.id(),
        summary: Some(StateSummary::from(summary)),
    };
    lobby_client
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
