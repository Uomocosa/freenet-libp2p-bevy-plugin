use freenet_stdlib::prelude::*;

use crate::Lobby;
use crate::LobbyContract;
use crate::constants;

pub fn contract_interface_validate_state(
    _parameters: Parameters<'static>,
    state: State<'static>,
    _related: RelatedContracts<'static>,
) -> Result<ValidateResult, ContractError> {
    let bytes = state.into_bytes();
    let lobby: Lobby = bincode::deserialize(&bytes).map_err(|_| ContractError::InvalidState)?;
    if lobby.len() > constants::MAX_ROOMS {
        return Err(ContractError::InvalidState);
    }
    for (room, record) in lobby.iter() {
        if !LobbyContract::valid_room_name(room.as_str()) {
            return Err(ContractError::InvalidState);
        }
        if record.members.len() > constants::MAX_MEMBERS_PER_ROOM {
            return Err(ContractError::InvalidState);
        }
    }
    Ok(ValidateResult::Valid)
}

#[cfg(test)]
mod tests {
    use super::contract_interface_validate_state;

    #[test]
    fn test_usage() {
        let _ = contract_interface_validate_state;
    }
}
