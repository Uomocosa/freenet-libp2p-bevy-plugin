use crate::direct_dial;

pub const fn close(state: &mut direct_dial::FriendsDrawerState) {
    let direct_dial::FriendsDrawerState(current) = state;
    *current = false;
}

#[cfg(test)]
mod tests {
    use crate::direct_dial;

    #[test]
    fn test_usage() {
        let mut state = direct_dial::FriendsDrawerState(true);
        state.close();
        assert!(!*state);
        state.close();
        assert!(!*state);
    }
}
