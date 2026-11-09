use std::collections::HashMap;

use libp2p::request_response::ResponseChannel;

use crate::p2p;

#[derive(Default)]
pub struct PendingSnapshotReplies {
    pending: HashMap<p2p::ReplyId, ResponseChannel<p2p::SnapshotResponse>>,
    next_id: u64,
}

#[rustfmt::skip]
impl PendingSnapshotReplies {
    pub fn insert(&mut self, channel: ResponseChannel<p2p::SnapshotResponse>) -> p2p::ReplyId {
        self.next_id = self.next_id.saturating_add(1);
        self.pending.insert(p2p::ReplyId(self.next_id), channel);
        p2p::ReplyId(self.next_id)
    }

    pub fn remove(&mut self, id: p2p::ReplyId) -> Option<ResponseChannel<p2p::SnapshotResponse>> {
        self.pending.remove(&id)
    }
}

// no test_usage necessary
