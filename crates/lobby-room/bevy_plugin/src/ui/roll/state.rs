use crate::ui;
use atomic_delegate_macros::atomic_delegates;

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub numbers: Vec<u32>,
    pub target: Vec<u32>,
    pub phase: ui::roll::Phase,
    pub reveal_slot: Option<usize>,
}

impl State {
    #[must_use]
    pub fn new(numbers: Vec<u32>, reveal_slot: Option<usize>) -> Self {
        Self {
            target: numbers.clone(),
            numbers,
            phase: ui::roll::Phase::Idle,
            reveal_slot,
        }
    }
}

impl State {
    pub fn update(&mut self, input: ui::roll::Input) -> Vec<ui::roll::Output> {
        match input {
            ui::roll::Input::Show { numbers } => self.show(numbers),
            ui::roll::Input::Tick { delta_secs } => self.tick(delta_secs),
        }
    }

    pub fn show(&mut self, numbers: Vec<u32>) -> Vec<ui::roll::Output> {
        self.target = numbers;
        self.align()
    }

    #[must_use]
    pub fn slot_value(&self, slot: usize) -> u32 {
        match &self.phase {
            ui::roll::Phase::Reveal {
                revealed_numbers, ..
            } => revealed_numbers.get(slot).copied().unwrap_or(0),
            _ => self.numbers.get(slot).copied().unwrap_or(0),
        }
    }

    #[must_use]
    pub fn reveal_progress(&self) -> f32 {
        match &self.phase {
            ui::roll::Phase::Reveal { elapsed_secs, .. } => {
                (elapsed_secs / ui::roll::REVEAL_SECS).clamp(0.0, 1.0)
            }
            _ => 1.0,
        }
    }
}

#[atomic_delegates]
impl State {
    pub(crate) fn align(&mut self) -> Vec<ui::roll::Output> {}
    pub(crate) fn begin_reveal(&mut self) -> Vec<ui::roll::Output> {}
    pub(crate) fn begin_roll(&mut self) -> Vec<ui::roll::Output> {}
    pub(crate) fn tick(&mut self, delta_secs: f32) -> Vec<ui::roll::Output> {}
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use ui::roll::{Input, Output, Phase};

    use super::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(vec![0, 8, 0], Some(2));
        let outputs = state.update(Input::Show {
            numbers: vec![0, 8, 1],
        });
        assert!(matches!(
            outputs.as_slice(),
            [Output::RevealStarted { slot: 2, value: 1 }]
        ));
        assert!(matches!(state.phase, Phase::Reveal { slot: 2, .. }));
    }

    #[test]
    fn show_while_idle_rolls_to_the_new_numbers() {
        let mut state = State::new(vec![2, 8, 1], Some(2));
        let _ = state.show(vec![3, 8, 1]);
        assert!(matches!(
            &state.phase,
            Phase::Roll { from, to, .. } if from == &vec![2, 8, 1] && to == &vec![3, 8, 1]
        ));
    }

    #[test]
    fn equal_numbers_never_start_an_animation() {
        let mut state = State::new(vec![2, 8, 1], Some(2));
        let outputs = state.show(vec![2, 8, 1]);
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.phase, Phase::Idle);
    }

    #[test]
    fn slot_value_shows_the_revealed_number_while_revealing() {
        let mut state = State::new(vec![2, 8, 0], Some(2));
        let _ = state.show(vec![2, 8, 1]);
        assert_eq!(state.slot_value(2), 1);
        assert_eq!(state.slot_value(0), 2);
    }

    #[test]
    fn reveal_progress_is_complete_when_idle() {
        let state = State::new(vec![2, 8, 1], Some(2));
        assert_eq!(state.reveal_progress(), 1.0);
    }

    #[test]
    fn a_friend_leaving_rolls_down_without_a_reveal() {
        let mut state = State::new(vec![2, 8, 1], Some(2));
        let _ = state.show(vec![2, 8, 0]);
        assert!(matches!(
            &state.phase,
            Phase::Roll { from, to, .. } if from == &vec![2, 8, 1] && to == &vec![2, 8, 0]
        ));
    }
}
