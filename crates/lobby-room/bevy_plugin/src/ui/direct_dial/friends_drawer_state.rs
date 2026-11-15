use atomic_delegate_macros::atomic_delegates;
use derive_more::{Deref, DerefMut};

use bevy::prelude::Resource;

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct FriendsDrawerState(pub bool);

#[atomic_delegates]
impl FriendsDrawerState {
    pub fn toggle(&mut self) {}
    pub fn close(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::FriendsDrawerState;

    #[test]
    fn test_usage() {
        let mut state = FriendsDrawerState::default();
        assert!(!*state);
        state.toggle();
        assert!(*state);
        state.close();
        assert!(!*state);
    }

    #[test]
    fn toggle_flips_the_open_state() {
        let mut state = FriendsDrawerState(false);
        state.toggle();
        assert!(*state);
        state.toggle();
        assert!(!*state);
    }

    #[test]
    fn close_shuts_an_open_drawer() {
        let mut state = FriendsDrawerState(true);
        state.close();
        assert!(!*state);
        state.close();
        assert!(!*state);
    }
}
