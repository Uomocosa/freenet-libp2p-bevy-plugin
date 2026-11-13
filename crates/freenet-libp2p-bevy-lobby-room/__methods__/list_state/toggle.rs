use crate::ui;

pub const fn toggle(state: &mut ui::roster::ListState) {
    let ui::roster::ListState(current) = state;
    *current = !*current;
}

#[cfg(test)]
mod tests {
    use crate::ui;

    #[test]
    fn test_usage() {
        let mut state = ui::roster::ListState::default();
        state.toggle();
        assert!(*state);
    }
}
