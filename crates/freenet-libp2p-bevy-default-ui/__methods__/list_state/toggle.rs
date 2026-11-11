use crate::roster;

pub const fn toggle(state: &mut roster::ListState) {
    let roster::ListState(current) = state;
    *current = !*current;
}

#[cfg(test)]
mod tests {
    use crate::roster;

    #[test]
    fn test_usage() {
        let mut state = roster::ListState::default();
        state.toggle();
        assert!(*state);
    }
}
