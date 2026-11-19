use std::sync::Arc;

use atomic_delegate_macros::atomic_delegates;
use freenet_libp2p_bevy_core::freenet_node;
use freenet_stdlib::prelude::{ContractCode, ContractKey, Parameters, WrappedContract};

use crate::discovery;
use crate::testing;

pub struct LobbyContractDeployer;

#[atomic_delegates]
impl LobbyContractDeployer {
    pub async fn ensure_exists(
        &self,
        token: &testing::harness::LobbyToken,
    ) -> Result<Vec<freenet_node::NodeDir>, testing::harness::LobbySetupError> {
    }
}

impl LobbyContractDeployer {
    pub(crate) fn lobby_key(params: &[u8]) -> ContractKey {
        let code = Arc::new(ContractCode::from(discovery::LOBBY_CONTRACT_WASM.to_vec()));
        let wrapped = WrappedContract::new(code, Parameters::from(params.to_vec()));
        wrapped.key
    }
}

#[cfg(test)]
mod tests {
    use super::LobbyContractDeployer;
    use crate::identity;

    #[test]
    fn test_usage() {
        let params = identity::GameId::from("lobby-e2e/ensured").contract_params();
        let first = LobbyContractDeployer::lobby_key(&params);
        assert_eq!(first, LobbyContractDeployer::lobby_key(&params));
        let other =
            LobbyContractDeployer::lobby_key(&identity::GameId::from("other").contract_params());
        assert_ne!(first, other);
        let _ = LobbyContractDeployer::ensure_exists;
    }
}
