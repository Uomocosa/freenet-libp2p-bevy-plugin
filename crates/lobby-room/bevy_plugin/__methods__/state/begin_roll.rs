use crate::ui;

pub fn begin_roll(state: &mut ui::roll::State) -> Vec<ui::roll::Output> {
    let from = state.numbers.clone();
    let to = state.target.clone();
    state.phase = ui::roll::Phase::Roll {
        from: from.clone(),
        to: to.clone(),
        elapsed_secs: 0.0,
    };
    vec![ui::roll::Output::RollFrame {
        from,
        to,
        progress: 0.0,
    }]
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use ui::roll::{Output, Phase, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(vec![2, 8, 1], Some(2));
        state.target = vec![3, 8, 1];
        let outputs = state.begin_roll();
        assert!(matches!(
            outputs.as_slice(),
            [Output::RollFrame { from, to, progress: 0.0 }] if from == &vec![2, 8, 1] && to == &vec![3, 8, 1]
        ));
        assert!(matches!(
            &state.phase,
            Phase::Roll { from, to, .. } if from == &vec![2, 8, 1] && to == &vec![3, 8, 1]
        ));
    }
}
