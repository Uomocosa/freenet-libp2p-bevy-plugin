use crate::discovery;

pub const fn toggle(state: &mut discovery::EscMenuState) {
    let discovery::EscMenuState(current) = state;
    *current = !*current;
}

#[cfg(test)]
mod tests {
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut state = discovery::EscMenuState(false);
        state.toggle();
        assert!(*state);
        state.toggle();
        assert!(!*state);
    }
}
