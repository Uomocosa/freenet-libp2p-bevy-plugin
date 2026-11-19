use std::sync::Arc;

use freenet_stdlib::prelude::{ContractCode, ContractKey, Parameters, WrappedContract};

#[must_use]
pub fn contract_key() -> ContractKey {
    WrappedContract::new(
        Arc::new(ContractCode::from(vec![0; 4])),
        Parameters::from(Vec::new()),
    )
    .key
}

#[cfg(test)]
mod tests {
    use super::contract_key;

    #[test]
    fn test_usage() {
        assert_eq!(contract_key(), contract_key());
    }
}
