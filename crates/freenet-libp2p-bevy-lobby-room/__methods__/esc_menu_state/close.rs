use crate::ui;

pub const fn close(state: &mut ui::discovery::EscMenuState) {
    let ui::discovery::EscMenuState(current) = state;
    *current = false;
}

#[cfg(test)]
mod tests {
    use crate::ui;

    #[test]
    fn test_usage() {
        let mut state = ui::discovery::EscMenuState(true);
        state.close();
        assert!(!*state);
    }
}
