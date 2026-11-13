use crate::ui;

pub const fn close(state: &mut ui::direct_dial::FriendsDrawerState) {
    let ui::direct_dial::FriendsDrawerState(current) = state;
    *current = false;
}

#[cfg(test)]
mod tests {
    use crate::ui;

    #[test]
    fn test_usage() {
        let mut state = ui::direct_dial::FriendsDrawerState(true);
        state.close();
        assert!(!*state);
        state.close();
        assert!(!*state);
    }
}
