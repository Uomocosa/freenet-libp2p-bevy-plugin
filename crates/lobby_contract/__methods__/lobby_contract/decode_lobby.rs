use crate::Lobby;
use freenet_stdlib::prelude::ContractError;

pub fn decode_lobby(bytes: &[u8]) -> Result<Lobby, ContractError> {
    let deserialization: Result<Lobby, _> = bincode::deserialize(bytes);
    deserialization.map_err(|error| ContractError::InvalidUpdateWithInfo {
        reason: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::decode_lobby;
    use crate::Lobby;

    #[test]
    fn test_usage() {
        let bytes = bincode::serialize(&Lobby::default()).unwrap();
        assert_eq!(decode_lobby(&bytes).unwrap(), Lobby::default());
        assert!(decode_lobby(&[]).is_err());
    }
}
