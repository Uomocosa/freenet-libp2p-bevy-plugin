use crate::discovery;

pub const fn close(state: &mut discovery::EscMenuState) {
    let discovery::EscMenuState(current) = state;
    *current = false;
}

#[cfg(test)]
mod tests {
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut state = discovery::EscMenuState(true);
        state.close();
        assert!(!*state);
    }
}
