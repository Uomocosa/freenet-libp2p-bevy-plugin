use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct ListState(pub bool);

#[atomic_delegates]
impl ListState {
    pub fn toggle(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::ListState;

    #[test]
    fn test_usage() {
        let mut state = ListState::default();
        assert!(!*state);
        state.toggle();
        assert!(*state);
        state.toggle();
        assert!(!*state);
    }
}
