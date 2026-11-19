use freenet_stdlib::client_api::{ContractResponse, HostResponse};
use freenet_stdlib::prelude::ContractKey;

#[must_use]
pub fn contract_not_found_response(key: ContractKey) -> HostResponse {
    HostResponse::ContractResponse(ContractResponse::NotFound {
        instance_id: *key.id(),
    })
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::client_api::{ContractResponse, HostResponse};

    use crate::testing;

    use super::contract_not_found_response;

    #[test]
    fn test_usage() {
        let key = testing::contract_key();
        assert!(matches!(
            contract_not_found_response(key),
            HostResponse::ContractResponse(ContractResponse::NotFound { instance_id })
                if instance_id == *key.id()
        ));
    }
}
