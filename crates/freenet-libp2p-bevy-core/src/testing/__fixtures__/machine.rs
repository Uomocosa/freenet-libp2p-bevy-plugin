use crate::identity;
use crate::spectator;

#[must_use]
pub fn machine() -> spectator::Machine<u8> {
    spectator::Machine::new(identity::GameId::from("game"))
}

#[cfg(test)]
mod tests {
    use super::machine;

    #[test]
    fn test_usage() {
        let machine = machine();
        assert_eq!(machine.game.as_str(), "game");
        assert!(machine.sessions.is_empty());
    }
}
