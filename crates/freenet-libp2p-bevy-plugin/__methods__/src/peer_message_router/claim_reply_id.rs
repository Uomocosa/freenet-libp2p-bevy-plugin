use crate::identity;
use crate::p2p;
use crate::testing;

pub fn claim_reply_id(
    router: &mut testing::PeerMessageRouter,
    requester: &identity::PeerId,
) -> p2p::ReplyId {
    router.next_reply_id = router.next_reply_id.saturating_add(1);
    let id = p2p::ReplyId(router.next_reply_id);
    router
        .pending_snapshot_replies
        .insert(id, requester.clone());
    id
}

#[cfg(test)]
mod tests {
    use super::claim_reply_id;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let requester = identity::PeerId::from("a");
        let first = claim_reply_id(&mut router, &requester);
        let second = claim_reply_id(&mut router, &requester);
        assert!(second > first);
        assert_eq!(
            router.pending_snapshot_replies.get(&first),
            Some(&requester)
        );
    }
}
