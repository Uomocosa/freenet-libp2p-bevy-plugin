use crate::direct_dial;

pub const fn toggle(state: &mut direct_dial::FriendsDrawerState) {
    let direct_dial::FriendsDrawerState(current) = state;
    *current = !*current;
}

#[cfg(test)]
mod tests {
    use crate::direct_dial;

    #[test]
    fn test_usage() {
        let mut state = direct_dial::FriendsDrawerState(false);
        state.toggle();
        assert!(*state);
        state.toggle();
        assert!(!*state);
    }
}
