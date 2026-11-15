use crate::ui;

pub const fn toggle(state: &mut ui::direct_dial::FriendsDrawerState) {
    let ui::direct_dial::FriendsDrawerState(current) = state;
    *current = !*current;
}

#[cfg(test)]
mod tests {
    use crate::ui;

    #[test]
    fn test_usage() {
        let mut state = ui::direct_dial::FriendsDrawerState(false);
        state.toggle();
        assert!(*state);
        state.toggle();
        assert!(!*state);
    }
}
