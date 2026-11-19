use freenet_stdlib::prelude::UpdateModification;

#[must_use]
pub fn state_bytes(modification: UpdateModification<'_>) -> Vec<u8> {
    modification
        .new_state
        .map_or_else(Vec::new, |state| state.as_ref().to_vec())
}

#[cfg(test)]
mod tests {
    use super::state_bytes;
    use freenet_stdlib::prelude::State;
    use freenet_stdlib::prelude::UpdateModification;

    #[test]
    fn test_usage() {
        let bytes = state_bytes(UpdateModification::valid(State::from(vec![1u8, 2, 3])));
        assert_eq!(bytes, vec![1u8, 2, 3]);
    }
}
