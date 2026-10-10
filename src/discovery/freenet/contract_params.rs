use crate::discovery;

#[must_use]
pub fn contract_params(game: &discovery::GameName, token: &discovery::GameToken) -> Vec<u8> {
    format!("{}/{}", game.as_str(), token.as_str()).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::contract_params;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let game = discovery::GameName::from("chess");
        let token = discovery::GameToken::from("token");
        assert_eq!(contract_params(&game, &token), b"chess/token".to_vec());
        let other = discovery::GameToken::from("other");
        assert_ne!(
            contract_params(&game, &token),
            contract_params(&game, &other)
        );
    }
}
