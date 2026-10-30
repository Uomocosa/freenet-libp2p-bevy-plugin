use std::collections::BTreeMap;

use atomic_delegate_macros::atomic_delegates;

use crate::identity;
use crate::room_sync;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MemberSnapshotFetchState {
    pub total_parts: u32,
    pub received_parts: BTreeMap<u32, Vec<u8>>,
    pub members: Vec<identity::PeerId>,
    pub asked_members: Vec<identity::PeerId>,
    pub current_source: Option<identity::PeerId>,
    pub part_requested_at: Option<room_sync::ElapsedSinceAppStart>,
}

#[atomic_delegates]
impl MemberSnapshotFetchState {
    pub fn join_received_parts(&self) -> Vec<u8> {}
}

#[cfg(test)]
mod tests {
    use super::MemberSnapshotFetchState;

    #[test]
    fn test_usage() {
        let state = MemberSnapshotFetchState::default();
        assert_eq!(state.join_received_parts(), Vec::<u8>::new());
    }
}
