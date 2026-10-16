use crate::identity;

#[must_use]
pub fn contract_params(game: &identity::GameId) -> Vec<u8> {
    game.as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::contract_params;
    use crate::identity;

    #[test]
    fn test_usage() {
        let game = identity::GameId::from("chess/token");
        assert_eq!(contract_params(&game), b"chess/token".to_vec());
        let other = identity::GameId::from("chess/other");
        assert_ne!(contract_params(&game), contract_params(&other));
    }
}
