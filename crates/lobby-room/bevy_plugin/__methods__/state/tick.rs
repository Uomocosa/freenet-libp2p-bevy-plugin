use crate::ui;

pub fn tick(state: &mut ui::roll::State, delta_secs: f32) -> Vec<ui::roll::Output> {
    match state.phase.clone() {
        ui::roll::Phase::Idle => Vec::new(),
        ui::roll::Phase::Reveal {
            slot,
            revealed_numbers,
            elapsed_secs,
        } => {
            let elapsed = elapsed_secs + delta_secs;
            if elapsed < ui::roll::REVEAL_SECS {
                state.phase = ui::roll::Phase::Reveal {
                    slot,
                    revealed_numbers,
                    elapsed_secs: elapsed,
                };
                return vec![ui::roll::Output::RevealFrame {
                    slot,
                    progress: elapsed / ui::roll::REVEAL_SECS,
                }];
            }
            let value = revealed_numbers.get(slot).copied().unwrap_or(0);
            state.numbers = revealed_numbers;
            state.phase = ui::roll::Phase::Idle;
            let mut outputs = vec![ui::roll::Output::RevealSettled { slot, value }];
            outputs.extend(state.align());
            outputs
        }
        ui::roll::Phase::Roll {
            from,
            to,
            elapsed_secs,
        } => {
            let elapsed = elapsed_secs + delta_secs;
            if elapsed < ui::roll::ROLL_SECS {
                state.phase = ui::roll::Phase::Roll {
                    from: from.clone(),
                    to: to.clone(),
                    elapsed_secs: elapsed,
                };
                return vec![ui::roll::Output::RollFrame {
                    from,
                    to,
                    progress: elapsed / ui::roll::ROLL_SECS,
                }];
            }
            state.numbers.clone_from(&to);
            state.phase = ui::roll::Phase::Idle;
            let mut outputs = vec![ui::roll::Output::RollSettled { numbers: to }];
            outputs.extend(state.align());
            outputs
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use ui::roll::{Input, Output, Phase, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(vec![2, 8, 0], Some(2));
        assert_eq!(state.update(Input::Tick { delta_secs: 0.05 }), Vec::new());
        assert_eq!(state.phase, Phase::Idle);
    }

    #[test]
    fn a_second_show_during_a_reveal_rolls_once_at_the_end() {
        let mut state = State::new(vec![2, 8, 0], Some(2));
        let _ = state.update(Input::Show {
            numbers: vec![2, 8, 1],
        });
        let _ = state.update(Input::Show {
            numbers: vec![3, 8, 2],
        });
        assert!(matches!(
            &state.phase,
            Phase::Reveal { revealed_numbers, .. } if revealed_numbers == &vec![2, 8, 1]
        ));
        assert_eq!(state.target, vec![3, 8, 2]);
        assert_eq!(state.numbers, vec![2, 8, 0]);
        for _ in 0..3 {
            let _ = state.update(Input::Tick { delta_secs: 0.05 });
        }
        assert!(matches!(&state.phase, Phase::Reveal { .. }));
        let outputs = state.update(Input::Tick { delta_secs: 0.45 });
        assert_eq!(state.numbers, vec![2, 8, 1]);
        assert!(matches!(
            &state.phase,
            Phase::Roll { from, to, .. } if from == &vec![2, 8, 1] && to == &vec![3, 8, 2]
        ));
        assert!(matches!(
            outputs.first(),
            Some(Output::RevealSettled { slot: 2, value: 1 })
        ));
        assert_eq!(
            outputs
                .iter()
                .filter(|output| matches!(output, Output::RollFrame { .. }))
                .count(),
            1
        );
        let outputs = state.update(Input::Tick { delta_secs: 0.30 });
        assert_eq!(state.phase, Phase::Idle);
        assert_eq!(state.numbers, vec![3, 8, 2]);
        assert!(matches!(
            outputs.as_slice(),
            [Output::RollSettled { numbers }] if numbers == &vec![3, 8, 2]
        ));
    }

    #[test]
    fn a_second_show_during_a_roll_is_rolled_once_at_the_end() {
        let mut state = State::new(vec![2, 8, 1], Some(2));
        let _ = state.show(vec![3, 8, 1]);
        let _ = state.show(vec![3, 8, 2]);
        assert!(matches!(&state.phase, Phase::Roll { to, .. } if to == &vec![3, 8, 1]));
        let outputs = state.update(Input::Tick { delta_secs: 0.30 });
        assert!(matches!(
            &state.phase,
            Phase::Roll { from, to, .. } if from == &vec![3, 8, 1] && to == &vec![3, 8, 2]
        ));
        assert_eq!(
            outputs
                .iter()
                .filter(|output| matches!(output, Output::RollFrame { .. }))
                .count(),
            1
        );
        let _ = state.update(Input::Tick { delta_secs: 0.30 });
        assert_eq!(state.phase, Phase::Idle);
        assert_eq!(state.numbers, vec![3, 8, 2]);
    }

    #[test]
    fn show_while_revealing_only_updates_the_target() {
        let mut state = State::new(vec![2, 8, 0], Some(2));
        let _ = state.show(vec![2, 8, 1]);
        let _ = state.show(vec![3, 8, 2]);
        assert!(matches!(
            &state.phase,
            Phase::Reveal { revealed_numbers, .. } if revealed_numbers == &vec![2, 8, 1]
        ));
        assert_eq!(state.target, vec![3, 8, 2]);
        assert_eq!(state.numbers, vec![2, 8, 0]);
    }

    #[test]
    fn a_reveal_settles_before_any_roll_starts() {
        let mut state = State::new(vec![2, 8, 0], Some(2));
        let _ = state.show(vec![2, 8, 1]);
        let _ = state.show(vec![3, 8, 2]);
        let frame = state.update(Input::Tick { delta_secs: 0.05 });
        assert!(matches!(frame.as_slice(), [Output::RevealFrame { .. }]));
        let settled = state.update(Input::Tick { delta_secs: 0.45 });
        assert!(matches!(
            settled.first(),
            Some(Output::RevealSettled { .. })
        ));
    }
}
