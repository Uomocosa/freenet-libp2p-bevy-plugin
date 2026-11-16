use std::process::ExitStatus;

use crate::testing;

pub fn exit_status(peer: &mut testing::harness::Peer) -> Result<Option<ExitStatus>, String> {
    peer.child
        .try_wait()
        .map_err(|e| peer.explain(&format!("checking the app process: {e}")))
}

// no test_usage necessary
