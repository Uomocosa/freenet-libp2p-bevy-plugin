use crate::ui;

pub fn align(state: &mut ui::roll::State) -> Vec<ui::roll::Output> {
    if state.phase != ui::roll::Phase::Idle || state.target == state.numbers {
        return Vec::new();
    }
    let crosses_reveal = state.reveal_slot.is_some_and(|slot| {
        state.numbers.get(slot).is_some_and(|&current| current == 0)
            && state.target.get(slot).is_some_and(|&next| next > 0)
    });
    if crosses_reveal {
        state.begin_reveal()
    } else {
        state.begin_roll()
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use ui::roll::{Output, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(vec![2, 8, 1], Some(2));
        state.target = vec![3, 8, 1];
        let outputs = state.align();
        assert!(matches!(
            outputs.as_slice(),
            [Output::RollFrame { progress: 0.0, .. }]
        ));
    }
}
