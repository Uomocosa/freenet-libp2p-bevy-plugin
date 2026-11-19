use crate::p2p;

pub fn log_mesh_snapshot<Payload: p2p::traits::Message>(state: &p2p::SwarmLoopState<Payload>) {
    let total = state.swarm.behaviour().gossipsub.all_mesh_peers().count();
    let mut topics = Vec::new();
    for topic in state.swarm.behaviour().gossipsub.topics() {
        let depth = state.swarm.behaviour().gossipsub.mesh_peers(topic).count();
        topics.push(format!("{topic}={depth}"));
    }
    tracing::debug!(target: "p2p", total, topics = ?topics, "p2p mesh snapshot");
}

#[cfg(test)]
mod tests {
    use super::log_mesh_snapshot;

    use crate::testing;

    #[test]
    fn test_usage() {
        let (state, _rx) = testing::swarm_loop_state::<u32>().expect("swarm loop state");
        log_mesh_snapshot(&state);
        assert_eq!(
            state.swarm.behaviour().gossipsub.all_mesh_peers().count(),
            0
        );
    }
}
