use crate::testing;

pub fn drop(peer: &mut testing::harness::Peer) {
    if matches!(peer.child.try_wait(), Ok(None)) {
        let _ = peer.child.kill();
        let _ = peer.child.wait();
    }
}

// no test_usage necessary
