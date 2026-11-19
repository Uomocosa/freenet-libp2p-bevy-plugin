use crate::identity;

#[must_use]
pub fn peek_game() -> identity::GameId {
    identity::GameId::from("game")
}

#[cfg(test)]
mod tests {
    use crate::identity;

    use super::peek_game;

    #[test]
    fn test_usage() {
        assert_eq!(peek_game(), identity::GameId::from("game"));
    }
}
