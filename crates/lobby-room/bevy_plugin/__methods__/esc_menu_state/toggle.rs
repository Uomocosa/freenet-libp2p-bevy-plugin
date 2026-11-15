use crate::ui;

pub const fn toggle(state: &mut ui::discovery::EscMenuState) {
    let ui::discovery::EscMenuState(current) = state;
    *current = !*current;
}

#[cfg(test)]
mod tests {
    use crate::ui;

    #[test]
    fn test_usage() {
        let mut state = ui::discovery::EscMenuState(false);
        state.toggle();
        assert!(*state);
        state.toggle();
        assert!(!*state);
    }
}
