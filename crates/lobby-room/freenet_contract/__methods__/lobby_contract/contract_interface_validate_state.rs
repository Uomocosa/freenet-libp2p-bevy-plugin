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
    use freenet_stdlib::prelude::{ContractError, Parameters, RelatedContracts, ValidateResult};

    use super::contract_interface_validate_state;
    use crate::Lobby;
    use crate::RoomName;
    use crate::UnixTime;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut lobby = Lobby::default();
        lobby.insert(
            RoomName::from("room-a"),
            testing::room("peer", UnixTime::from_secs(5), UnixTime::from_secs(10)),
        );
        let validated = contract_interface_validate_state(
            Parameters::from(Vec::new()),
            testing::state_of(&lobby),
            RelatedContracts::default(),
        );
        assert!(matches!(validated, Ok(ValidateResult::Valid)));
    }

    #[test]
    fn a_room_name_outside_the_allowed_charset_is_invalid_state() {
        let mut lobby = Lobby::default();
        lobby.insert(
            RoomName::from("room a"),
            testing::room("peer", UnixTime::from_secs(5), UnixTime::from_secs(10)),
        );
        let validated = contract_interface_validate_state(
            Parameters::from(Vec::new()),
            testing::state_of(&lobby),
            RelatedContracts::default(),
        );
        assert!(matches!(validated, Err(ContractError::InvalidState)));
    }
}
