use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct EscMenuState(pub bool);

#[atomic_delegates]
impl EscMenuState {
    pub fn toggle(&mut self) {}
    pub fn close(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::EscMenuState;

    #[test]
    fn test_usage() {
        let mut state = EscMenuState::default();
        assert!(!*state);
        state.toggle();
        assert!(*state);
        state.close();
        assert!(!*state);
    }

    #[test]
    fn toggle_flips_the_open_state() {
        let mut state = EscMenuState(false);
        state.toggle();
        assert!(*state);
        state.toggle();
        assert!(!*state);
    }

    #[test]
    fn close_shuts_an_open_menu() {
        let mut state = EscMenuState(true);
        state.close();
        assert!(!*state);
        state.close();
        assert!(!*state);
    }
}
