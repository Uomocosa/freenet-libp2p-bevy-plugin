use crate::ui;

pub fn begin_reveal(state: &mut ui::roll::State) -> Vec<ui::roll::Output> {
    let Some(slot) = state.reveal_slot else {
        return Vec::new();
    };
    let Some(&value) = state.target.get(slot) else {
        return Vec::new();
    };
    state.phase = ui::roll::Phase::Reveal {
        slot,
        revealed_numbers: state.target.clone(),
        elapsed_secs: 0.0,
    };
    vec![ui::roll::Output::RevealStarted { slot, value }]
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use ui::roll::{Output, Phase, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(vec![0, 8, 0], Some(2));
        state.target = vec![0, 8, 1];
        let outputs = state.begin_reveal();
        assert!(matches!(
            outputs.as_slice(),
            [Output::RevealStarted { slot: 2, value: 1 }]
        ));
        assert!(matches!(
            &state.phase,
            Phase::Reveal { revealed_numbers, .. } if revealed_numbers == &vec![0, 8, 1]
        ));
    }
}
