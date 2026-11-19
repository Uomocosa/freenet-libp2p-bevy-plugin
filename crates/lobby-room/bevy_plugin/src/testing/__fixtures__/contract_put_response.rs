use freenet_stdlib::client_api::{ContractResponse, HostResponse};
use freenet_stdlib::prelude::ContractKey;

#[must_use]
pub const fn contract_put_response(key: ContractKey) -> HostResponse {
    HostResponse::ContractResponse(ContractResponse::PutResponse { key })
}

#[cfg(test)]
mod tests {
    use freenet_stdlib::client_api::{ContractResponse, HostResponse};

    use crate::testing;

    use super::contract_put_response;

    #[test]
    fn test_usage() {
        let key = testing::contract_key();
        assert!(matches!(
            contract_put_response(key),
            HostResponse::ContractResponse(ContractResponse::PutResponse { key: found })
                if found == key
        ));
    }
}
