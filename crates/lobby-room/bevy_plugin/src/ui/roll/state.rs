use crate::ui;

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

#[cfg(test)]
mod tests {
    use crate::ui;

    use super::State;

    #[test]
    fn test_usage() {
        let state = State::new(vec![2, 8, 0], Some(2));
        assert_eq!(state.numbers, vec![2, 8, 0]);
        assert_eq!(state.target, vec![2, 8, 0]);
        assert_eq!(state.phase, ui::roll::Phase::Idle);
    }
}
