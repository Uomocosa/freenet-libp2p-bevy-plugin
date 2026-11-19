use freenet_stdlib::client_api::{ContractResponse, HostResponse};
use freenet_stdlib::prelude::{ContractKey, WrappedState};

#[must_use]
pub fn contract_get_response<S: serde::Serialize>(key: ContractKey, state: &S) -> HostResponse {
    HostResponse::ContractResponse(ContractResponse::GetResponse {
        key,
        contract: None,
        state: WrappedState::new(bincode::serialize(state).unwrap_or_default()),
    })
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::client_api::{ContractResponse, HostResponse};

    use crate::testing;

    use super::contract_get_response;

    #[test]
    fn test_usage() {
        let key = testing::contract_key();
        assert!(matches!(
            contract_get_response(key, &testing::TestState(7)),
            HostResponse::ContractResponse(ContractResponse::GetResponse { key: found, .. })
                if found == key
        ));
    }
}
