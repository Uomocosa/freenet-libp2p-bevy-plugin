use freenet_stdlib::prelude::State;

use crate::Lobby;

#[must_use]
pub fn state_of(lobby: &Lobby) -> State<'static> {
    State::from(bincode::serialize(lobby).expect("a lobby serializes"))
}

#[cfg(test)]
mod tests {
    use super::state_of;
    use crate::Lobby;

    #[test]
    fn test_usage() {
        let state = state_of(&Lobby::default());
        assert_eq!(
            state.as_ref(),
            bincode::serialize(&Lobby::default())
                .expect("a lobby serializes")
                .as_slice()
        );
    }
}
