use crate::p2p;

pub fn log_mesh_snapshot<Payload: p2p::Message>(state: &p2p::SwarmLoopState<Payload>) {
    let total = state.swarm.behaviour().gossipsub.all_mesh_peers().count();
    let mut topics = Vec::new();
    for topic in state.swarm.behaviour().gossipsub.topics() {
        let depth = state.swarm.behaviour().gossipsub.mesh_peers(topic).count();
        topics.push(format!("{topic}={depth}"));
    }
    tracing::debug!(target: "p2p", total, topics = ?topics, "p2p mesh snapshot");
}

// no test_usage necessary

#[cfg(test)]
mod tests {
    use super::log_mesh_snapshot;

    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        log_mesh_snapshot(&state);
    }
}
