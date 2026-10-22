use crate::room_sync;

#[must_use]
pub fn join_received_parts(state: &room_sync::state_machine::MemberSnapshotFetchState) -> Vec<u8> {
    let mut bytes = Vec::new();
    for part in state.received_parts.values() {
        bytes.extend_from_slice(part);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::join_received_parts;
    use crate::room_sync;

    #[test]
    fn test_usage() {
        let state = room_sync::state_machine::MemberSnapshotFetchState::default();
        assert_eq!(join_received_parts(&state), Vec::<u8>::new());
    }
}
